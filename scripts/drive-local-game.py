"""Recorded Universal Modder input with physical client pixels on scaled Windows.

Uses the pinned UM WinDrive safety checks; a private copy adds per-monitor DPI
awareness so capture pixels and click coordinates agree. Small 0.5 captures use
coordinates multiplied by2. The user must have authorized driving the game.
"""
import argparse
import hashlib
import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path

from um import win

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--run", type=Path, required=True)
parser.add_argument("commands", nargs="+")
args = parser.parse_args()
lab = Path(__file__).resolve().parents[2] / "lab"
run = args.run.resolve()
if not run.is_relative_to((lab / "runs").resolve()) or not run.is_dir():
    parser.error("--run must be an existing lab run directory")
source = Path(win.tool_path("WinDrive.ps1")).read_text(encoding="utf-8-sig")
old = '[DllImport("user32.dll")] static extern uint SendInput'
new = '[DllImport("user32.dll")] static extern IntPtr SetThreadDpiAwarenessContext(IntPtr context);\n    ' + old
if source.count(old) != 1 or source.count('        proc = p.EndsWith') != 1:
    raise ValueError("Universal Modder input source differs from expected pinned helper")
derived = source.replace(old, new).replace('        proc = p.EndsWith', '        SetThreadDpiAwarenessContext(new IntPtr(-4));\n        proc = p.EndsWith')
derived = derived.replace('            Console.Out.WriteLine(reply);', '            Thread.Sleep(150);\n            Console.Out.WriteLine(reply);')
helper = lab / "tools/WinDrive-Dpi.ps1"
helper.write_text(derived, encoding="utf-8")
record = {"time_utc": datetime.now(timezone.utc).isoformat(), "process": "FreeRealms",
          "commands": args.commands, "coordinate_space": "physical client pixels",
          "um_source_sha256": hashlib.sha256(source.encode()).hexdigest(),
          "helper_sha256": hashlib.sha256(derived.encode()).hexdigest()}
startup = subprocess.STARTUPINFO()
startup.dwFlags |= subprocess.STARTF_USESHOWWINDOW
result = subprocess.run([win.ps_exe(), "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", str(helper), "-Proc", "FreeRealms"],
                        input="\n".join(args.commands) + "\n", text=True, capture_output=True,
                        startupinfo=startup, creationflags=subprocess.CREATE_NO_WINDOW, timeout=60)
record.update(exit_code=result.returncode, stdout=result.stdout, stderr=result.stderr)
with (run / "external-inputs.jsonl").open("a", encoding="utf-8", newline="\n") as handle:
    handle.write(json.dumps(record) + "\n")
print(result.stdout, end="")
if result.returncode or "error " in result.stdout or result.stderr:
    raise RuntimeError("game input failed; see recorded run input log")
