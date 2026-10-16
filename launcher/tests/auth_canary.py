"""Identity-pin mutation gate. Run with python -I launcher/tests/auth_canary.py.

Requires the broker binaries built in lab/build/w100/server first. A failing
baseline is a blocker, never a successful canary. No private values are printed.
"""
import os
from pathlib import Path
import subprocess
import sys

project = Path(__file__).resolve().parents[2]
output = project.parent / "lab/build/w100"
output.mkdir(parents=True, exist_ok=True)
source = project / "launcher/src/auth.rs"
environment = dict(os.environ, CARGO_TARGET_DIR=str(output / "launcher"))


def run(name):
    log = output / name
    with log.open("wb") as stream:
        result = subprocess.run(
            ["powershell.exe", "-NoProfile", "-File", str(project / "launcher/build.ps1"), "test"],
            cwd=project, env=environment, stdout=stream, stderr=subprocess.STDOUT,
            timeout=240,
        )
    return result.returncode, log.read_text(errors="replace")


baseline, _ = run("canary-baseline.log")
if baseline:
    sys.exit("BLOCKED: baseline failed; pin canary was not run. See canary-baseline.log.")

original = source.read_bytes()
needle = b"if !bool::from(challenge[1..33].ct_eq(&profile.server_identity)) {"
assert original.count(needle) == 1, "pin condition changed; review canary"
mutated = original.replace(needle, b"if false && !bool::from(challenge[1..33].ct_eq(&profile.server_identity)) {")
try:
    source.write_bytes(mutated)
    code, log = run("canary-mutated.log")
    assert code != 0 and "wrong_pin_sends_no_proof_canary ... FAILED" in log, "pin bypass survived its test"
finally:
    # This file is W100-owned. Refuse to overwrite an unexpected concurrent edit.
    assert source.read_bytes() == mutated, "concurrent auth.rs edit; restore manually"
    source.write_bytes(original)
print("PASS: bypassing the identity pin failed wrong_pin_sends_no_proof_canary; source restored.")
