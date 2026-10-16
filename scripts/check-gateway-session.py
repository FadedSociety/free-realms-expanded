"""Opt-in real-UDP gateway session regressions; private outputs stay in lab/runs.

Run with the private Python runtime and --run. No application/game payloads are
sent or recorded. All subprocesses are bounded and cleaned up by exact handle.
"""

import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import socket
import struct
import subprocess
import time
import uuid
import zlib


def sha256(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def session_request(session_id, crc_length=3):
    # The compiled serializer's fixed CGAPI_527 synthetic request layout.
    return struct.pack(">HIII", 1, crc_length, session_id, 512) + b"CGAPI_527\0"


def response_seed(wire, session_id):
    require(len(wire) == 21, "session response must contain 21 bytes")
    require(wire[:2] == b"\0\2", "session response opcode differs")
    require(int.from_bytes(wire[2:6], "big") == session_id, "session ID differs")
    require(wire[10:13] == bytes([3, 1, 0]), "negotiated CRC/compression differs")
    require(int.from_bytes(wire[13:17], "big") == 512, "buffer size differs")
    require(int.from_bytes(wire[17:21], "big") == 3, "session trailer differs")
    return int.from_bytes(wire[6:10], "big")


def ping(seed):
    # Opcode 6, uncompressed flag 0, then the low three CRC bytes. Python's
    # independent zlib implementation checks the Rust seeded CRC convention.
    content = b"\0\6\0"
    crc = zlib.crc32(seed.to_bytes(4, "little") + content)
    return content + crc.to_bytes(4, "big")[-3:]


def receive(sock, peer):
    wire, sender = sock.recvfrom(65535)
    require(sender == peer, "received a datagram from an unexpected peer")
    return wire


def listening_address(process, stdout_path, timeout=5.0):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        for line in stdout_path.read_text(encoding="utf-8").splitlines():
            try:
                event = json.loads(line)
            except json.JSONDecodeError:
                continue
            if event.get("event") == "listening":
                host, port = event["address"].rsplit(":", 1)
                require(host == "127.0.0.1", "gateway listening address is not loopback")
                return host, int(port)
        require(process.poll() is None, "gateway exited before announcing its socket")
        time.sleep(0.02)
    raise RuntimeError("gateway did not announce its socket within five seconds")


def main():
    project = Path(__file__).resolve().parents[1]
    lab = project.parent / "lab"
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--run", action="store_true", help="explicitly launch the bounded checks")
    parser.add_argument(
        "--gateway", type=Path,
        default=project / "prototype/osfr-tools/target/release/osfr-gateway.exe",
    )
    parser.add_argument(
        "--reference", type=Path, default=lab / "references/rust-gateway-reference",
    )
    args = parser.parse_args()
    if not args.run:
        parser.error("this checker launches a local process; opt in with --run")
    gateway = args.gateway.resolve(strict=True)
    reference = args.reference.resolve(strict=True)
    require(gateway.is_file(), "gateway executable is missing")
    require((reference / "manifest.json").is_file(), "private reference manifest is missing")
    run_name = "gateway-session-check-" + datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    run_name += "-" + uuid.uuid4().hex[:8]
    output = lab / "runs" / run_name
    output.mkdir(parents=True, exist_ok=False)
    report = {
        "status": "running",
        "started_utc": datetime.now(timezone.utc).isoformat(),
        "gateway": str(gateway),
        "gateway_sha256": sha256(gateway),
        "reference_manifest_sha256": sha256(reference / "manifest.json"),
        "script_sha256": sha256(Path(__file__)),
        "scope": "synthetic session/control UDP only; no game payloads",
        "checks": [],
        "processes": [],
    }
    processes = []

    def launch(label, bind, seconds):
        stdout_path = output / (label + "-stdout.jsonl")
        stderr_path = output / (label + "-stderr.log")
        state = output / (label + "-character.json")
        command = [
            str(gateway), bind, "--reference", str(reference), "--state", str(state),
            "--seconds", str(seconds),
        ]
        with stdout_path.open("x", encoding="utf-8") as stdout, stderr_path.open(
            "x", encoding="utf-8"
        ) as stderr:
            process = subprocess.Popen(
                command, stdin=subprocess.DEVNULL, stdout=stdout, stderr=stderr,
                creationflags=subprocess.CREATE_NO_WINDOW if os.name == "nt" else 0,
            )
        metadata = {"label": label, "pid": process.pid, "command": command}
        processes.append((process, metadata))
        report["processes"].append(metadata)
        return process, stdout_path, stderr_path, state

    def passed(name, **metadata):
        report["checks"].append({"name": name, "passed": True, **metadata})

    try:
        # Parse-time rejection must happen before creating a state or socket.
        rejected, _, stderr_path, rejected_state = launch("nonloopback", "0.0.0.0:0", 2)
        require(rejected.wait(timeout=5) != 0, "nonloopback bind was accepted")
        require("only binds loopback" in stderr_path.read_text(encoding="utf-8"),
                "nonloopback bind did not report the expected reason")
        require(not rejected_state.exists(), "rejected bind unexpectedly created state")
        passed("nonloopback_bind_rejected")

        process, stdout_path, _, _ = launch("gateway", "127.0.0.1:0", 12)
        peer = listening_address(process, stdout_path)
        report["address"] = f"{peer[0]}:{peer[1]}"
        with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as sock:
            sock.bind(("127.0.0.1", 0))
            sock.settimeout(2.0)
            first_id = 0x10203040
            first_request = session_request(first_id)
            sock.sendto(first_request, peer)
            first_response = receive(sock, peer)
            first_seed = response_seed(first_response, first_id)
            sock.sendto(first_request, peer)
            duplicate_response = receive(sock, peer)
            require(duplicate_response == first_response, "duplicate handshake response changed")
            passed("duplicate_handshake_response_identical",
                   response_sha256=hashlib.sha256(first_response).hexdigest())

            sock.sendto(session_request(first_id + 1, crc_length=0), peer)
            # Current server resends the retained session's response to its peer.
            retained_response = receive(sock, peer)
            require(retained_response == first_response,
                    "rejected replacement did not retain the prior session response")
            sock.sendto(ping(first_seed), peer)
            require(receive(sock, peer) == b"\0\6",
                    "valid old-session ping failed after rejected replacement")
            passed("crc_zero_replacement_does_not_evict_active_session")

            # Deliberately choose an ID that would cancel the two generation
            # increments in the former seed ^ request_id ^ generation formula
            # (the rejected replacement also consumes one generation). This
            # makes the distinct-seed check a regression for that exact bug.
            base_seed = 1_884_358_976
            old_formula_generation = first_seed ^ base_seed ^ first_id
            second_id = first_id ^ old_formula_generation ^ ((old_formula_generation + 2) & 0xFFFFFFFF)
            require(second_id != first_id, "replacement test ID did not change")
            sock.sendto(session_request(second_id), peer)
            second_response = receive(sock, peer)
            second_seed = response_seed(second_response, second_id)
            require(second_seed != first_seed, "new session reused the prior CRC seed")
            require(ping(first_seed) != ping(second_seed), "test pings have colliding CRC trailers")
            passed("replacement_session_has_different_seed",
                   first_seed=first_seed, second_seed=second_seed,
                   first_session_id=first_id, second_session_id=second_id)

            sock.settimeout(0.4)
            sock.sendto(ping(first_seed), peer)
            try:
                unexpected = receive(sock, peer)
            except socket.timeout:
                passed("old_seed_ping_ignored", observation_seconds=0.4)
            else:
                raise RuntimeError(
                    "stale-session ping produced a response of " + str(len(unexpected)) + " bytes"
                )
            sock.settimeout(2.0)
            sock.sendto(ping(second_seed), peer)
            require(receive(sock, peer) == b"\0\6", "new-session ping failed")
            passed("new_seed_ping_works")

        require(process.wait(timeout=15) == 0, "bounded gateway exited with an error")
        events = [json.loads(line) for line in stdout_path.read_text(encoding="utf-8").splitlines()]
        names = [event.get("event") for event in events]
        require(names.count("session_started") == 2, "unexpected number of successful sessions")
        require(names.count("session_rejected") == 1, "CRC-zero replacement was not logged as rejected")
        require(names[-1] == "stopped", "gateway did not finish its bounded run cleanly")
        passed("bounded_gateway_stops_cleanly", event_names=names)
        report["status"] = "passed"
    except Exception as error:
        report["status"] = "failed"
        report["error"] = f"{type(error).__name__}: {error}"
    finally:
        # Only terminate processes launched here, using their original handles.
        for process, metadata in processes:
            if process.poll() is None:
                process.terminate()
                metadata["terminated_by_checker"] = True
                try:
                    process.wait(timeout=2)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait(timeout=2)
                    metadata["killed_by_checker"] = True
            metadata["exit_code"] = process.returncode
            metadata["running_after_cleanup"] = process.poll() is None
        report["completed_utc"] = datetime.now(timezone.utc).isoformat()
        report_path = output / "report.json"
        report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
        print(json.dumps({"status": report["status"], "report": str(report_path)}))
    return 0 if report["status"] == "passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
