"""Small offline acceptance check for friend_launcher.py."""

from __future__ import annotations

import importlib.util
import shutil
import subprocess
import sys
import uuid
from contextlib import contextmanager
from pathlib import Path


PROJECT = Path(__file__).resolve().parents[2]
CLIENT_SOURCE = PROJECT.parent / "lab/working/OSFR.Client/OSFR Client"
LAUNCHER = Path(__file__).with_name("friend_launcher.py")
SPEC = importlib.util.spec_from_file_location("friend_launcher", LAUNCHER)
launcher = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(launcher)


@contextmanager
def writable_temporary_directory():
    path = Path(__file__).resolve().parent / f".test-friend-launcher-{uuid.uuid4().hex}"
    path.mkdir()
    try:
        yield path
    finally:
        shutil.rmtree(path)


def main() -> None:
    with writable_temporary_directory() as scratch:
        client = scratch / "client"
        client.mkdir()
        for name in ("FreeRealms.bat", "ClientConfig.ini", "UserOptions.ini"):
            shutil.copy2(CLIENT_SOURCE / name, client / name)
        (client / "FreeRealms.exe").write_bytes(b"MZ test marker; never launched")
        (client / "locale").mkdir()
        for name in launcher.LOCALE_FILES:
            shutil.copy2(CLIENT_SOURCE / "locale" / name, client / "locale" / name)
        (client / "Baseline.ini").write_bytes((client / "ClientConfig.ini").read_bytes())
        original_client_config = (client / "ClientConfig.ini").read_bytes()

        locale_pair = scratch / "replacement-locale"
        locale_pair.mkdir()
        for name in launcher.LOCALE_FILES:
            shutil.copy2(CLIENT_SOURCE / "locale" / name, locale_pair / name)
            target = client / "locale" / name
            target.write_bytes(target.read_bytes() + b"previous test overlay")
        old = {path: path.read_bytes() for path in (client / "Baseline.ini", client / "UserOptions.ini", *(client / "locale" / n for n in launcher.LOCALE_FILES))}

        dry_run = subprocess.run(
            [
                sys.executable,
                "-I",
                str(LAUNCHER),
                str(client),
                "127.0.0.1:20260",
                "--locale-pair",
                str(locale_pair),
                "--dry-run",
            ],
            text=True,
            capture_output=True,
            check=True,
        )
        for path in old:
            assert path.read_bytes() == old[path], f"dry-run changed {path}"
            assert f"BACKUP {path}" in dry_run.stdout
            assert f"WRITE  {path}" in dry_run.stdout

        no_locale_plan = launcher.build_plan(client, "127.0.0.1:20260")
        assert all("locale" not in target.parts for target in no_locale_plan["changes"])
        plan = launcher.build_plan(client, "127.0.0.1:20260", locale_pair)
        backup_dir = launcher.apply_plan(plan)
        assert backup_dir is not None
        baseline = (client / "Baseline.ini").read_text(encoding="utf-8")
        assert "Server=127.0.0.1:20260" in baseline
        assert "IndirectServerAddress=http://127.0.0.1:9/" in baseline
        assert "http://osfr.editz.co.uk/assets" not in baseline
        assert (client / "ClientConfig.ini").read_bytes() == original_client_config
        options = (client / "UserOptions.ini").read_text(encoding="utf-8")
        for key, value in launcher.USER_OPTIONS.items():
            assert f"{key}={value}" in options
        for name in launcher.LOCALE_FILES:
            target = client / "locale" / name
            assert target.read_bytes() == (locale_pair / name).read_bytes()
            assert (backup_dir / "locale" / name).read_bytes() == old[target]
        for name in ("Baseline.ini", "UserOptions.ini"):
            assert (backup_dir / name).read_bytes() == old[client / name]
        arguments = plan["arguments"]
        assert "inifile=Baseline.ini" in arguments
        assert "Server=127.0.0.1:20260" in arguments
        assert not any(arg.lower().startswith(("ticket=", "key=", "guid=", "cassessionid=")) for arg in arguments)
        changed = {"Baseline.ini", "UserOptions.ini", "locale/en_us_data.dat", "locale/en_us_data.dir"}
        for path in client.rglob("*"):
            if path.is_file() and not path.is_relative_to(backup_dir):
                relative = path.relative_to(client).as_posix()
                assert relative in changed or relative in {"FreeRealms.exe", "FreeRealms.bat", "ClientConfig.ini"}
        assert launcher.validate_endpoint("[::1]:20042") == "[::1]:20042"
        for endpoint in ("host:0", "host:65536", "host:20042;whoami", "http://host:20042"):
            try:
                launcher.validate_endpoint(endpoint)
            except ValueError:
                pass
            else:
                raise AssertionError(f"unsafe endpoint accepted: {endpoint}")

        not_client = scratch / "not-a-client"
        not_client.mkdir()
        (not_client / "FreeRealms.exe").write_bytes(b"not a Windows executable")
        try:
            launcher.build_plan(not_client, "127.0.0.1:20260")
        except (OSError, ValueError):
            pass
        else:
            raise AssertionError("non-client folder was accepted")
        print("DRY-RUN file plan:")
        plan_lines = [
            line.replace(str(scratch), "<temp>")
            for line in dry_run.stdout.splitlines()
            if line.startswith(("BACKUP", "WRITE"))
        ]
        print("\n".join(plan_lines))
    print("PASS: dry-run listed config/locale writes and backups without changing files")
    print("PASS: config, settings, native-login arguments, locale install and backups verified")
    print("PASS: omitted locale pair leaves locale files out of the write plan")
    print("PASS: non-client folder refused; client executable was not launched")


if __name__ == "__main__":
    main()
