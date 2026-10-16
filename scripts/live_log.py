"""Merge a Free Realms run's packet, service, and client logs in timestamp order."""
import argparse
import ctypes
import json
import os
import re
import sys
import time
from ctypes import wintypes
from datetime import datetime
from pathlib import Path


FIXED_LOGS = (
    "server-trace.jsonl", "login-trace.jsonl", "native-hook.jsonl",
    "server-stdout.log", "server-stderr.log",
    "login-stdout.log", "login-stderr.log",
    "client-stdout.log", "client-stderr.log",
)
SENSITIVE_KEYS = {
    "name", "player", "playername", "charactername", "firstname", "lastname",
    "username", "chat", "chattext", "text", "ticket", "key", "handoffkey",
    "secret", "identity",
}
FIELD_RE = re.compile(
    r"(?i)(\b(?:player[-_ ]?name|character[-_ ]?name|first[-_ ]?name|last[-_ ]?name|"
    r"user[-_ ]?name|chat(?:[-_ ]?text)?|ticket|handoff[-_ ]?key|secret|identity|name)\s*[:=]\s*)"
    r"(\"[^\"]*\"|'[^']*'|[^,;\]}\r\n]+)"
)
TIMESTAMP_RE = re.compile(
    r"^(?:\[)?(\d{4}-\d\d-\d\d[ T]\d\d:\d\d:\d\d(?:\.\d+)?(?:Z|[+-]\d\d:\d\d)?|"
    r"\d\d:\d\d:\d\d(?:\.\d+)?)"
)


def discover_sources(run):
    paths = [run / name for name in FIXED_LOGS]
    client_logs = run / "client" / "Logs"
    if client_logs.is_dir():
        paths.extend(p for p in client_logs.rglob("*") if p.is_file() and p.suffix.lower() == ".log")
    client = run / "client"
    if client.is_dir():
        paths.extend(p for p in run.rglob("*Probe*.log") if p.is_file())
    seen = set()
    return [p for p in paths if p.is_file() and not (str(p.resolve()).lower() in seen or seen.add(str(p.resolve()).lower()))]


def _sensitive_key(key):
    if key.lower() in {"family_name", "soe_name"}:
        return False
    words = re.sub(r"([a-z0-9])([A-Z])", r"\1 \2", key)
    return bool(set(re.findall(r"[a-z0-9]+", words.lower())) & SENSITIVE_KEYS)


def sanitize(value, key=""):
    if _sensitive_key(key):
        return "[redacted]"
    if isinstance(value, dict):
        return {k: sanitize(v, k) for k, v in value.items()}
    if isinstance(value, list):
        return [sanitize(v) for v in value]
    if isinstance(value, str):
        return FIELD_RE.sub(lambda m: m.group(1) + "[redacted]", value)
    return value


def _timestamp(record, raw, fallback):
    for key in ("wall_ms", "timestamp_ms", "time_ms"):
        value = record.get(key) if isinstance(record, dict) else None
        if isinstance(value, (int, float)):
            return float(value)
    value = record.get("ts_ms") if isinstance(record, dict) else None
    if isinstance(value, (int, float)):
        return float(value)
    match = TIMESTAMP_RE.match(raw)
    if match:
        value = match.group(1).replace(" ", "T", 1)
        if len(value) >= 10 and value[4] == "-":
            try:
                return datetime.fromisoformat(value.replace("Z", "+00:00")).timestamp() * 1000
            except ValueError:
                pass
        else:
            parts = value.split(":")
            if len(parts) == 3:
                try:
                    time_ms = (int(parts[0]) * 3600 + int(parts[1]) * 60 + float(parts[2])) * 1000
                    if isinstance(fallback, (int, float)) and fallback > 1_000_000_000_000:
                        day = int(fallback // 86_400_000) * 86_400_000
                        time_ms += day
                        if time_ms - fallback > 43_200_000:
                            time_ms -= 86_400_000
                        elif fallback - time_ms > 43_200_000:
                            time_ms += 86_400_000
                    return time_ms
                except ValueError:
                    pass
    return fallback


def _severity(record, raw):
    fields = record if isinstance(record, dict) else {}
    outcome = str(fields.get("outcome", "")).lower()
    event = str(fields.get("event", "")).lower()
    line = raw.lower()
    if outcome in {"rejected", "rejected_buffered", "unhandled"} or "unhandled" in event:
        return "error"
    if outcome in {"ignored", "clamped"}:
        return "warning"
    if any(word in line for word in ("error", "exception", "failed", "rejected", "unhandled")):
        return "error"
    if "warning" in line or "clamp" in line:
        return "warning"
    return "info"


def _record(raw):
    try:
        value = json.loads(raw)
    except (json.JSONDecodeError, TypeError):
        return None
    return sanitize(value) if isinstance(value, dict) else None


def _summary(source, record, raw):
    if record is None:
        return FIELD_RE.sub(lambda m: m.group(1) + "[redacted]", raw.strip())
    event = str(record.get("event", "log"))
    family, subtype = record.get("family"), record.get("subtype")
    if isinstance(family, int) and isinstance(subtype, int):
        name = record.get("family_name") or ""
        summary = f"{family}/{subtype} {name}".rstrip()
    elif isinstance(record.get("login_opcode"), int):
        summary = f"login opcode {record['login_opcode']}"
    else:
        summary = event
    transport = record.get("transport")
    if isinstance(transport, dict):
        name = transport.get("soe_name") or transport.get("soe_opcode")
        seq = transport.get("reliable_seq")
        summary += f" SOE {name}"
        if seq is not None:
            summary += f" seq={seq}"
        if transport.get("mode"):
            summary += f" {transport['mode']}"
    for key in ("handler", "movement", "outcome", "reason"):
        value = record.get(key)
        if value not in (None, ""):
            summary += f" {key}={value}"
    if event == "native_hook":
        summary = f"NATIVE {record.get('label')}"
        values = record.get("values") or {}
        if values:
            summary += " " + " ".join(f"{k}={v}" for k, v in values.items())
        if record.get("error"):
            summary += f" error={record['error']}"
        return summary
    if event == "app_out":
        summary += f" len={record.get('body_len')}"
        if record.get("body_head_hex"):
            summary += f" hex={record['body_head_hex']}"
        causes = [
            f"{c.get('family')}/{c.get('subtype')}" if c.get("family") is not None else str(c.get("handler"))
            for c in record.get("cause") or []
        ]
        if causes:
            summary += " cause=" + ",".join(causes)
        if record.get("recipient") is not None:
            summary += f" to={record['recipient']}"
    if event.startswith(("npc_", "quest_", "offer_")):
        for key in ("guid", "content_key", "option", "quest_id", "variant", "accepted", "outcome", "reason"):
            if record.get(key) is not None and f" {key}=" not in summary:
                summary += f" {key}={record[key]}"
        if isinstance(record.get("options"), list):
            summary += " options=" + ",".join(str(o.get("action")) for o in record["options"] if isinstance(o, dict))
    direction = record.get("direction")
    if direction == "in":
        summary = "<- " + summary
    elif direction == "out":
        summary = "-> " + summary
    return summary


def read_all(run):
    events = []
    sources = discover_sources(run)
    for order, path in enumerate(sources):
        source = str(path.relative_to(run))
        fallback = path.stat().st_mtime_ns // 1_000_000
        try:
            lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
        except OSError:
            continue
        for line_no, raw in enumerate(lines):
            record = _record(raw)
            events.append({
                "sort": _timestamp(record, raw, fallback),
                "source_order": order,
                "line": line_no,
                "source": source,
                "record": record,
                "raw": raw,
                "severity": _severity(record, raw),
                "summary": _summary(source, record, raw),
            })
    events.sort(key=lambda item: (item["sort"], item["source_order"], item["line"]))
    return events


def _render(item):
    stamp = item["sort"]
    record = item["record"]
    if record and isinstance(record.get("wall_ms"), (int, float)):
        stamp = datetime.fromtimestamp(record["wall_ms"] / 1000).strftime("%H:%M:%S.%f")[:-3]
    elif record and isinstance(record.get("ts_ms"), (int, float)):
        stamp = f"+{record['ts_ms']:.1f}ms"
    elif isinstance(stamp, (int, float)) and stamp > 1_000_000_000_000:
        stamp = datetime.fromtimestamp(stamp / 1000).strftime("%H:%M:%S.%f")[:-3]
    elif isinstance(stamp, (int, float)):
        stamp = f"{stamp:.3f}ms"
    else:
        stamp = str(stamp)
    return f"{stamp} [{item['source']}] {item['summary']}"


def _matches(item, only, hide, grep):
    if only:
        family = item["record"].get("family") if item["record"] else None
        if family not in only:
            return False
    searchable = (item["summary"] + " " + item["source"] + " " + item["raw"]).lower()
    if any(value in searchable for value in hide):
        return False
    return not grep or grep.lower() in searchable


def _color(item, text):
    if not sys.stdout.isatty():
        return text
    if item["severity"] == "error":
        code = "31;1"
    elif item["severity"] == "warning":
        code = "33"
    else:
        source = item["source"].lower()
        code = "36" if "login" in source else "32" if "client" in source else "34" if "server" in source else "37"
    return f"\x1b[{code}m{text}\x1b[0m"


def self_test(run):
    events = read_all(run)
    assert events, f"no log events found in {run}"
    present = {Path(item["source"]).name for item in events}
    assert any(name in present for name in ("server-stdout.log", "server-trace.jsonl", "login-stdout.log", "login-trace.jsonl")), present
    for item in events:
        record = item["record"]
        if record and record.get("family") == 15 and record.get("subtype") == 1:
            assert record.get("body_head_hex") in (None, ""), "chat body prefix was recorded"
        assert _privacy_ok(record), item["source"]
    print(json.dumps({"ok": True, "events": len(events), "sources": sorted(present), "timestamp_ordered": True}))


def _privacy_ok(value, key=""):
    if _sensitive_key(key):
        return value == "[redacted]"
    if isinstance(value, dict):
        return all(_privacy_ok(child, name) for name, child in value.items())
    if isinstance(value, list):
        return all(_privacy_ok(child) for child in value)
    return True


def _windows_position(client_pid):
    if os.name != "nt" or not client_pid:
        return
    user32, kernel32 = ctypes.windll.user32, ctypes.windll.kernel32
    kernel32.GetConsoleWindow.restype = wintypes.HWND
    kernel32.SetConsoleTitleW.argtypes = [wintypes.LPCWSTR]
    user32.GetWindowRect.argtypes = [wintypes.HWND, ctypes.POINTER(wintypes.RECT)]
    user32.GetWindowRect.restype = wintypes.BOOL
    user32.IsWindowVisible.argtypes = [wintypes.HWND]
    user32.IsWindowVisible.restype = wintypes.BOOL
    user32.GetWindowThreadProcessId.argtypes = [wintypes.HWND, ctypes.POINTER(wintypes.DWORD)]
    user32.GetWindowThreadProcessId.restype = wintypes.DWORD
    user32.GetSystemMetrics.argtypes = [ctypes.c_int]
    user32.GetSystemMetrics.restype = ctypes.c_int
    user32.SetWindowPos.argtypes = [wintypes.HWND, wintypes.HWND, ctypes.c_int, ctypes.c_int,
                                    ctypes.c_int, ctypes.c_int, wintypes.UINT]
    user32.SetWindowPos.restype = wintypes.BOOL
    console = kernel32.GetConsoleWindow()
    if not console:
        return
    client_rect = wintypes.RECT()
    found = {"hwnd": None}
    callback_type = ctypes.WINFUNCTYPE(ctypes.c_bool, wintypes.HWND, wintypes.LPARAM)
    user32.EnumWindows.argtypes = [callback_type, wintypes.LPARAM]
    user32.EnumWindows.restype = wintypes.BOOL

    def inspect(hwnd, _):
        pid = wintypes.DWORD()
        user32.GetWindowThreadProcessId(hwnd, ctypes.byref(pid))
        if pid.value == client_pid and user32.IsWindowVisible(hwnd) and user32.GetWindowRect(hwnd, ctypes.byref(client_rect)):
            found["hwnd"] = hwnd
            return False
        return True

    callback = callback_type(inspect)
    for _ in range(30):
        user32.EnumWindows(callback, 0)
        if found["hwnd"]:
            break
        time.sleep(0.25)
    if not found["hwnd"]:
        return
    console_rect = wintypes.RECT()
    if not user32.GetWindowRect(console, ctypes.byref(console_rect)):
        return
    width = max(480, console_rect.right - console_rect.left)
    height = client_rect.bottom - client_rect.top
    x = client_rect.right
    if x + width > user32.GetSystemMetrics(0):
        x = max(0, client_rect.left - width)
    user32.SetWindowPos(console, wintypes.HWND(0), x, client_rect.top, width, height, 0x0040 | 0x0010)  # SHOWWINDOW | NOACTIVATE: never steal focus from the game


def _parent_alive(pid):
    if not pid:
        return True
    if os.name == "nt":
        # Never use os.kill(pid, 0) on Windows: it calls TerminateProcess, and an
        # access error made the viewer think the launcher had exited.
        kernel32 = ctypes.windll.kernel32
        handle = kernel32.OpenProcess(0x1000, False, pid)  # QUERY_LIMITED_INFORMATION
        if not handle:
            return False
        try:
            code = ctypes.c_ulong()
            if not kernel32.GetExitCodeProcess(handle, ctypes.byref(code)):
                return True
            return code.value == 259  # STILL_ACTIVE
        finally:
            kernel32.CloseHandle(handle)
    try:
        os.kill(pid, 0)
        return True
    except OSError:
        return False


def _log_error(path, error, raw):
    try:
        with path.open("a", encoding="utf-8") as handle:
            handle.write(f"{type(error).__name__}: {error} :: {str(raw)[:200]!r}\n")
    except OSError:
        pass


def tail(run, args):
    if os.name == "nt":
        ctypes.windll.kernel32.SetConsoleTitleW("Free Realms Live Log")
        _windows_position(args.client_pid)
    try:
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    except (AttributeError, ValueError):
        pass
    output = (run / "live.log").open("a", encoding="utf-8", buffering=1)
    errors = run / "live-log-error.txt"
    positions, pending, known = {}, {}, set()
    idle_after_parent = None
    try:
        while True:
            events = []
            for path in discover_sources(run):
                key = str(path.resolve())
                known.add(key)
                try:
                    size = path.stat().st_size
                    offset = positions.get(key, 0)
                    if size < offset:
                        offset = 0
                        pending[key] = ""
                    with path.open("r", encoding="utf-8", errors="replace") as handle:
                        handle.seek(offset)
                        chunk = handle.read()
                        positions[key] = handle.tell()
                except OSError:
                    continue
                text = pending.get(key, "") + chunk
                lines = text.splitlines(keepends=True)
                pending[key] = ""
                if lines and not lines[-1].endswith(("\n", "\r")):
                    pending[key] = lines.pop()
                for line_no, raw in enumerate(lines):
                    raw = raw.rstrip("\r\n")
                    source = str(path.relative_to(run))
                    # One malformed line must never kill the live window.
                    try:
                        record = _record(raw)
                        events.append({
                            "sort": _timestamp(record, raw, time.time_ns() // 1_000_000),
                            "source_order": source,
                            "line": line_no,
                            "source": source,
                            "record": record,
                            "raw": raw,
                            "severity": _severity(record, raw),
                            "summary": _summary(source, record, raw),
                        })
                    except Exception as error:  # noqa: BLE001 - keep tailing
                        _log_error(errors, error, raw)

            def sort_key(item):
                stamp = item["sort"]
                number = stamp if isinstance(stamp, (int, float)) else float("inf")
                return (number, item["source_order"], item["line"])

            events.sort(key=sort_key)
            for item in events:
                try:
                    rendered = _render(item)
                    output.write(rendered + "\n")
                    if _matches(item, args.only, args.hide, args.grep):
                        print(_color(item, rendered), flush=True)
                except Exception as error:  # noqa: BLE001 - keep tailing
                    _log_error(errors, error, item.get("raw", ""))
            if not _parent_alive(args.parent_pid):
                if events:
                    idle_after_parent = time.monotonic()
                elif idle_after_parent is None:
                    idle_after_parent = time.monotonic()
                elif time.monotonic() - idle_after_parent > 0.75:
                    break
            time.sleep(0.10)
    except KeyboardInterrupt:
        pass
    finally:
        output.close()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("run_directory", type=Path)
    parser.add_argument("--only", help="Comma-separated game families, such as 26,49,66")
    parser.add_argument("--hide", default="", help="Comma-separated event or handler terms to hide")
    parser.add_argument("--grep", default="", help="Case-insensitive text filter")
    parser.add_argument("--self-test", action="store_true", help="Read and validate a recorded run without writing files")
    parser.add_argument("--client-pid", type=int, help=argparse.SUPPRESS)
    parser.add_argument("--parent-pid", type=int, help=argparse.SUPPRESS)
    args = parser.parse_args()
    run = args.run_directory.resolve()
    if not run.is_dir():
        parser.error(f"run directory does not exist: {run}")
    if args.self_test:
        self_test(run)
        return
    args.only = {int(value) for value in args.only.split(",") if value.strip()} if args.only else set()
    args.hide = [value.strip().lower() for value in args.hide.split(",") if value.strip()]
    tail(run, args)


if __name__ == "__main__":
    main()
