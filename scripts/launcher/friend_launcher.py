"""Prepare and launch a supplied Free Realms client against a LAN server.

Usage (Python 3.9+, standard library only):
    python friend_launcher.py "C:\\Games\\Free Realms" 192.168.1.20:20042
    python friend_launcher.py "C:\\Games\\Free Realms" 192.168.1.20:20042 --locale-pair "C:\\Share\\en_us_locale" --dry-run

The optional locale directory must contain en_us_data.dat and en_us_data.dir.
Backups go under .friend-launcher-backups in the client folder. --dry-run
prints all target and backup files without writing or launching. The launcher
does not download or execute anything from the network; it starts the supplied
local client with the validated Server endpoint.

Launch arguments follow the native-login path in scripts/run-local-baseline.py
(argument extraction at lines 235-259); the five UserOptions values and
Baseline.ini edits follow lines 224-234. The original harness points
IndirectServerAddress at its temporary asset server; this launcher has no such
service, so the shared loopback placeholder keeps that URL offline. The
client's own FreeRealms.bat supplies the remaining arguments.
"""

from __future__ import annotations

import argparse
import datetime as dt
import ipaddress
import os
import re
import shlex
import shutil
import subprocess
import sys
from pathlib import Path


VERSION = "1.0.0"
BACKUP_ROOT = ".friend-launcher-backups"
LOCALE_FILES = ("en_us_data.dat", "en_us_data.dir")
USER_OPTIONS = {
    "WindowedWidth": "1024",
    "WindowedHeight": "768",
    "Mode": "Windowed",
    "Maximized": "0",
    "Master": "0.000000",
}
AUTH_ARGUMENTS = {"guid", "ticket", "key", "cassessionid", "sessionid", "server"}
CLIENT_ARGUMENTS = {
    "inifile",
    "showmemberloadingscreen",
    "country",
    "internationalization:locale",
}


def validate_endpoint(value: str) -> str:
    """Accept an IP or DNS host followed by a valid TCP/UDP port, without lookup."""
    if value.startswith("["):
        close = value.find("]")
        host = value[1:close] if close > 1 else ""
        suffix = value[close + 1 :] if close >= 0 else ""
        if not suffix.startswith(":") or ":" in suffix[1:]:
            raise ValueError("server must be host:port; bracket IPv6, for example [::1]:20042")
        try:
            if ipaddress.ip_address(host).version != 6:
                raise ValueError
        except ValueError as exc:
            raise ValueError("bracketed server host must be an IPv6 address") from exc
        port_text = suffix[1:]
    else:
        host, separator, port_text = value.rpartition(":")
        if not separator or ":" in host:
            raise ValueError("server must be host:port; bracket IPv6, for example [::1]:20042")
        try:
            ipaddress.ip_address(host)
        except ValueError:
            labels = host.split(".")
            label_ok = all(
                re.fullmatch(r"[A-Za-z0-9](?:[A-Za-z0-9-]{0,61}[A-Za-z0-9])?", label)
                for label in labels
            )
            if not host or len(host) > 253 or not label_ok:
                raise ValueError("server host must be an IPv4 address or DNS name")
    if not port_text.isascii() or not port_text.isdecimal() or not 1 <= int(port_text) <= 65535:
        raise ValueError("server port must be between 1 and 65535")
    return value


def _regular_file(root: Path, relative: str) -> Path:
    path = root / relative
    if path.is_symlink() or not path.is_file():
        raise ValueError(f"not a regular client file: {relative}")
    if not path.resolve().is_relative_to(root):
        raise ValueError(f"client path escapes its folder: {relative}")
    return path


def validate_client(client_dir: Path) -> Path:
    root = client_dir.expanduser().resolve(strict=True)
    if not root.is_dir():
        raise ValueError(f"not a client folder: {client_dir}")
    executable = _regular_file(root, "FreeRealms.exe")
    with executable.open("rb") as handle:
        if handle.read(2) != b"MZ":
            raise ValueError("FreeRealms.exe is missing its Windows executable signature")
    for relative in ("FreeRealms.bat", "ClientConfig.ini", "UserOptions.ini"):
        _regular_file(root, relative)
    locale = root / "locale"
    if locale.is_symlink() or not locale.is_dir() or not locale.resolve().is_relative_to(root):
        raise ValueError("not a Free Realms client folder: locale directory is missing or unsafe")
    for name in LOCALE_FILES:
        _regular_file(root, f"locale/{name}")
    return root


def _render_baseline(root: Path, endpoint: str) -> str:
    text = (root / "ClientConfig.ini").read_text(encoding="utf-8-sig")
    pattern = re.compile(r"(?m)^Server=.*$")
    if len(pattern.findall(text)) != 1:
        raise ValueError("ClientConfig.ini must contain exactly one Server= line")
    text = pattern.sub("Server=" + endpoint, text, count=1)
    # Match the harness's offline URL rewrite. Its local asset server is not
    # part of this launcher, so IndirectServerAddress stays on the same sink.
    text = re.sub(r"https?://[^\s\r\n]+", "http://127.0.0.1:9/", text)
    text = re.sub(r"(?m)^Address=lvsadp[^\r\n]*", "Address=127.0.0.1", text)
    return text


def _render_user_options(root: Path) -> str:
    text = (root / "UserOptions.ini").read_text(encoding="utf-8-sig")
    for key, value in USER_OPTIONS.items():
        pattern = re.compile(rf"(?m)^{re.escape(key)}=.*$")
        if len(pattern.findall(text)) != 1:
            raise ValueError(f"UserOptions.ini must contain exactly one {key}= line")
        text = pattern.sub(f"{key}={value}", text, count=1)
    return text


def launch_arguments(root: Path, endpoint: str) -> list[str]:
    """Use the harness's native-login argument filtering and fixed overrides."""
    batch = (root / "FreeRealms.bat").read_text(encoding="utf-8-sig")
    matches = list(
        re.finditer(
            r"(?im)^\s*@?(?:start(?:\s+/[^\s]+)*\s+)?FreeRealms\.exe\s+([^\r\n]+)",
            batch,
        )
    )
    if len(matches) != 1:
        raise ValueError("FreeRealms.bat must contain exactly one FreeRealms.exe command line")
    arguments = shlex.split(matches[0].group(1), posix=False)
    unexpected = {
        arg.split("=", 1)[0].lower()
        for arg in arguments
        if arg.split("=", 1)[0].lower() not in AUTH_ARGUMENTS
        and arg.split("=", 1)[0].lower() not in CLIENT_ARGUMENTS
    }
    if unexpected:
        raise ValueError(
            "FreeRealms.bat has unsupported argument names: " + ", ".join(sorted(unexpected))
        )
    arguments = [
        "inifile=Baseline.ini" if arg.lower().startswith("inifile=") else arg
        for arg in arguments
    ]
    if not any(arg.lower().startswith("inifile=") for arg in arguments):
        raise ValueError("FreeRealms.bat command line has no inifile= argument")
    arguments = [
        arg
        for arg in arguments
        if arg.split("=", 1)[0].lower() not in AUTH_ARGUMENTS
    ]
    arguments.extend(
        [
            "Server=" + endpoint,
            "SessionId=local-session",
            "Internationalization:Locale=8",
            "GameAccountManagement:UriRoot=http://127.0.0.1:9/",
        ]
    )
    return arguments


def _safe_target(root: Path, relative: str) -> Path:
    path = root / relative
    if path.is_symlink() or (path.exists() and not path.is_file()):
        raise ValueError(f"unsafe launcher target: {relative}")
    if path.exists() and path.stat().st_nlink > 1:
        raise ValueError(f"refusing to overwrite a hard-linked file: {relative}")
    if not path.parent.resolve().is_relative_to(root):
        raise ValueError(f"launcher target escapes client folder: {relative}")
    return path


def build_plan(client_dir: Path, endpoint: str, locale_pair: Path | None = None) -> dict:
    endpoint = validate_endpoint(endpoint)
    root = validate_client(client_dir)
    changes = {
        _safe_target(root, "Baseline.ini"): _render_baseline(root, endpoint),
        _safe_target(root, "UserOptions.ini"): _render_user_options(root),
    }
    if locale_pair is not None:
        source_dir = locale_pair.expanduser().resolve(strict=True)
        if not source_dir.is_dir():
            raise ValueError("--locale-pair must name a folder containing the locale pair")
        for name in LOCALE_FILES:
            source = source_dir / name
            if source.is_symlink() or not source.is_file():
                raise ValueError(f"locale pair is missing a regular {name}")
            if source.stat().st_size > 64 * 1024 * 1024:
                raise ValueError(f"locale file is unexpectedly large: {name}")
            target = _safe_target(root, "locale/" + name)
            if source.resolve() != target.resolve():
                changes[target] = source

    stamp = dt.datetime.now(dt.timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
    backup_dir = root / BACKUP_ROOT / stamp
    backup_parent = root / BACKUP_ROOT
    if backup_parent.is_symlink() or (backup_parent.exists() and not backup_parent.is_dir()):
        raise ValueError(f"unsafe backup path: {backup_parent}")
    if not backup_parent.resolve().is_relative_to(root) or backup_dir.exists():
        raise ValueError("backup path is unsafe or already exists")
    backups = {
        target: backup_dir / target.relative_to(root)
        for target in changes
        if target.exists()
    }
    return {
        "root": root,
        "endpoint": endpoint,
        "changes": changes,
        "backup_dir": backup_dir,
        "backups": backups,
        "arguments": launch_arguments(root, endpoint),
    }


def print_plan(plan: dict) -> None:
    root = plan["root"]
    print(f"Client: {root}")
    for target, backup in plan["backups"].items():
        print(f"BACKUP {target} -> {backup}")
    for target in plan["changes"]:
        print(f"WRITE  {target}")
    print(f"LAUNCH {root / 'FreeRealms.exe'}")
    print("ARGS   " + " ".join(plan["arguments"]))


def apply_plan(plan: dict) -> Path | None:
    if plan["backups"]:
        backup_dir = plan["backup_dir"]
        backup_dir.mkdir(parents=True, exist_ok=False)
        for target, backup in plan["backups"].items():
            backup.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(target, backup)
    else:
        backup_dir = None
    for target, content in plan["changes"].items():
        if isinstance(content, Path):
            shutil.copy2(content, target)
        else:
            target.write_text(content, encoding="utf-8")
    return backup_dir


def launch_client(plan: dict) -> int:
    if os.name != "nt":
        raise OSError("launching Free Realms requires Windows")
    startup = subprocess.STARTUPINFO()
    startup.dwFlags |= subprocess.STARTF_USESHOWWINDOW
    startup.wShowWindow = 0
    process = subprocess.Popen(
        [str(plan["root"] / "FreeRealms.exe"), *plan["arguments"]],
        cwd=plan["root"],
        startupinfo=startup,
        creationflags=subprocess.CREATE_NO_WINDOW,
    )
    return process.pid


def main() -> int:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument("client_dir", type=Path, help="folder containing FreeRealms.exe")
    parser.add_argument("server", help="server host:port; bracket IPv6, for example [::1]:20042")
    parser.add_argument(
        "--locale-pair",
        type=Path,
        help="optional folder containing en_us_data.dat and en_us_data.dir",
    )
    parser.add_argument("--dry-run", action="store_true", help="print file changes; do not write or launch")
    parser.add_argument("--version", action="version", version=f"%(prog)s {VERSION}")
    args = parser.parse_args()
    try:
        plan = build_plan(args.client_dir, args.server, args.locale_pair)
        if args.dry_run:
            print_plan(plan)
            return 0
        if os.name != "nt":
            raise OSError("launching Free Realms requires Windows")
        backup_dir = apply_plan(plan)
        if backup_dir:
            print(f"Backups: {backup_dir}")
        print(f"Prepared client for {plan['endpoint']}")
        print(f"FreeRealms.exe PID {launch_client(plan)}")
        return 0
    except (OSError, ValueError) as exc:
        print(f"friend-launcher: error: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
