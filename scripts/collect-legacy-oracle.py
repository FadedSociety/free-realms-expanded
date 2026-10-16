"""Capture a bounded, synthetic login/ready exchange with a copied legacy server.

No original game client is started. Raw replies stay in the sibling analysis lab.
Run with the bundled Python, or another Python 3.10+ on Windows.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import socket
import struct
import subprocess
import time
import zlib
from datetime import datetime, timezone


CRC_SEED = 1_884_358_976
MAX_MESSAGE = 16 * 1024 * 1024


def crc(data: bytes, seed: int = CRC_SEED) -> int:
    # The legacy table routine feeds the seed least-significant byte first,
    # then the entire transport header/body. zlib supplies an independent CRC.
    return zlib.crc32(struct.pack("<I", seed) + data)


def frame(opcode: int, body: bytes) -> bytes:
    raw = struct.pack(">HB", opcode, 0) + body
    return raw + struct.pack(">H", crc(raw) & 0xFFFF)


def unpack(data: bytes) -> tuple[int, bytes, bool]:
    if len(data) < 5:
        raise ValueError("truncated CRC/compression frame")
    expected = crc(data[:-2]) & 0xFFFF
    if int.from_bytes(data[-2:], "big") != expected:
        raise ValueError("CRC mismatch (independently checked)")
    opcode = int.from_bytes(data[:2], "big")
    flag = data[2]
    if flag not in (0, 1):
        raise ValueError(f"unsupported compression flag {flag}")
    payload = data[3:-2]
    if flag:
        inflater = zlib.decompressobj()
        payload = inflater.decompress(payload, MAX_MESSAGE + 1)
        if len(payload) > MAX_MESSAGE or not inflater.eof or inflater.unused_data:
            raise ValueError("invalid, trailing, or oversized zlib stream")
    return opcode, payload, bool(flag)


class Capture:
    def __init__(self, sock: socket.socket, output: Path, peer: tuple[str, int]):
        self.sock, self.output, self.peer = sock, output, peer
        self.frames: list[dict] = []
        self.messages: list[dict] = []
        self.fragments: bytearray | None = None
        self.fragment_length = 0
        self.fragment_first = 0
        self.expected_sequence = 0

    def save_frame(self, direction: str, phase: str, data: bytes) -> dict:
        name = f"frames/{len(self.frames):05d}-{direction}-{phase}.bin"
        (self.output / name).write_bytes(data)
        record = {"path": name, "direction": direction, "phase": phase,
                  "length": len(data), "sha256": hashlib.sha256(data).hexdigest()}
        self.frames.append(record)
        return record

    def send(self, phase: str, data: bytes) -> None:
        self.save_frame("tx", phase, data)
        self.sock.sendto(data, self.peer)

    def message(self, phase: str, data: bytes, first: int, last: int) -> None:
        if len(data) < 2:
            raise ValueError("truncated application opcode")
        outer = int.from_bytes(data[:2], "little")
        record = {"phase": phase, "first_sequence": first, "last_sequence": last,
                  "length": len(data), "outer_opcode": outer,
                  "sha256": hashlib.sha256(data).hexdigest()}
        if outer in (5, 6):
            if len(data) < 9:
                raise ValueError("truncated application tunnel")
            inner_length = int.from_bytes(data[3:7], "little")
            if inner_length != len(data) - 7:
                raise ValueError("application tunnel length mismatch")
            record.update(tunnel_flag=data[2], inner_length=inner_length,
                          inner_opcode=int.from_bytes(data[7:9], "little"))
        name = f"messages/{len(self.messages):04d}-{phase}.bin"
        (self.output / name).write_bytes(data)
        record["path"] = name
        self.messages.append(record)

    def receive_phase(self, phase: str, timeout: float = 12) -> None:
        deadline = time.monotonic() + timeout
        last_data = time.monotonic()
        while time.monotonic() < deadline:
            try:
                data, address = self.sock.recvfrom(65535)
            except socket.timeout:
                if time.monotonic() - last_data > 1.5:
                    break
                continue
            if address != self.peer:
                raise ValueError(f"unexpected UDP peer {address}")
            last_data = time.monotonic()
            record = self.save_frame("rx", phase, data)
            opcode, payload, compressed = unpack(data)
            record.update(opcode=opcode, crc_verified=True, compressed=compressed,
                          expanded_body_length=len(payload))
            if opcode == 21:
                if len(payload) != 2:
                    raise ValueError("invalid acknowledge length")
                record["ack_sequence"] = int.from_bytes(payload, "big")
                continue
            if opcode not in (9, 13):
                raise ValueError(f"unexpected transport opcode {opcode}")
            if len(payload) < 2:
                raise ValueError("truncated data sequence")
            sequence = int.from_bytes(payload[:2], "big")
            record["sequence"] = sequence
            if sequence != self.expected_sequence:
                raise ValueError(f"sequence {sequence}, expected {self.expected_sequence}")
            self.expected_sequence = (sequence + 1) & 0xFFFF
            if opcode == 9:
                if self.fragments is not None:
                    raise ValueError("interleaved unfragmented application message")
                self.message(phase, payload[2:], sequence, sequence)
            else:
                if self.fragments is None:
                    if len(payload) < 6:
                        raise ValueError("truncated first fragment")
                    self.fragment_length = int.from_bytes(payload[2:6], "big")
                    if not 2 <= self.fragment_length <= MAX_MESSAGE:
                        raise ValueError("invalid or oversized fragmented message")
                    self.fragment_first = sequence
                    self.fragments = bytearray(payload[6:])
                else:
                    self.fragments.extend(payload[2:])
                if len(self.fragments) > self.fragment_length:
                    raise ValueError("fragment data exceeds declared size")
                if len(self.fragments) == self.fragment_length:
                    self.message(phase, bytes(self.fragments), self.fragment_first, sequence)
                    self.fragments = None
        if self.fragments is not None:
            raise ValueError("capture ended with incomplete fragmented message")
        # The supplied server sends a whole stream without a transmission
        # window and sleeps 13 ms after each inbound datagram. Per-fragment
        # ACKs would queue ahead of the next application request for seconds.
        # A single end-of-phase ACK is enough for this oracle; the server only
        # logs ACKs, so this does not establish real-client ACK semantics.
        if self.expected_sequence:
            self.send("ack", frame(21, struct.pack(">H", (self.expected_sequence - 1) & 0xFFFF)))


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--skip-ready", action="store_true")
    args = parser.parse_args()
    project = Path(__file__).resolve().parents[1]
    lab = project.parent / "lab"
    source = lab / "working" / "OSFR.Server" / "OSFR Server"
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
    output = lab / "captures" / f"legacy-gateway-{stamp}"
    run = lab / "runs" / f"legacy-gateway-{stamp}"
    output.mkdir(parents=True)
    (output / "frames").mkdir()
    (output / "messages").mkdir()
    # Original logs include old startup lines; use a new log directory so the
    # readiness check describes this exact process rather than a copied run.
    shutil.copytree(source, run, ignore=shutil.ignore_patterns("Logs"))
    (run / "Logs").mkdir()
    port_probe = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    port_probe.bind(("127.0.0.1", 0))
    port = port_probe.getsockname()[1]
    port_probe.close()
    config = json.loads((run / "Config.json").read_text(encoding="utf-8-sig"))
    config[0]["Port"] = port
    (run / "Config.json").write_text(json.dumps(config, indent=2), encoding="utf-8")
    report = {"recorded_utc": datetime.now(timezone.utc).isoformat(),
              "source_directory": str(source), "run_directory": str(run),
              "capture_directory": str(output), "port": port,
              "game_client_launched": False, "client_requests": "synthetic",
              "legacy_bind": "0.0.0.0 (unchanged legacy server behavior)",
              "server_sha256": hashlib.sha256((source / "OSFRServer.exe").read_bytes()).hexdigest(),
              "crc_seed": CRC_SEED, "crc_length": 2,
              "status": "started", "server_stopped": False}
    proc = None
    capture = None
    try:
        with (output / "server-stdout.log").open("wb") as stdout, \
                (output / "server-stderr.log").open("wb") as stderr, \
                socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as sock:
            sock.bind(("127.0.0.1", 0))
            sock.setsockopt(socket.SOL_SOCKET, socket.SO_RCVBUF, 4 * 1024 * 1024)
            sock.settimeout(0.15)
            proc = subprocess.Popen([str(run / "OSFRServer.exe")], cwd=run,
                                    stdout=stdout, stderr=stderr,
                                    creationflags=subprocess.CREATE_NO_WINDOW)
            report["pid"] = proc.pid
            startup_deadline = time.monotonic() + 20
            while time.monotonic() < startup_deadline:
                if proc.poll() is not None:
                    raise RuntimeError(f"server exited during startup: {proc.returncode}")
                log = run / "Logs" / "OSFR.log"
                if log.exists() and "Started listening!" in log.read_text(errors="replace"):
                    break
                time.sleep(0.1)
            else:
                raise TimeoutError("server did not reach Started listening within 20 seconds")
            capture = Capture(sock, output, ("127.0.0.1", port))
            session = 0x12345678
            request = struct.pack(">HIII", 1, 2, session, 512) + b"CGAPI_527\0"
            expected = struct.pack(">HII BBB II", 2, session, CRC_SEED, 2, 1, 0, 512, 3)
            capture.send("session", request)
            sock.settimeout(3)
            response, address = sock.recvfrom(65535)
            capture.save_frame("rx", "session", response)
            if address != capture.peer or response != expected:
                raise ValueError("session response differs from independently constructed expectation")
            sock.settimeout(0.15)
            # The legacy login handler does not read any fields after opcode 1.
            capture.send("login", frame(9, struct.pack(">H", 0) + b"\x01\x00"))
            capture.receive_phase("login")
            if not capture.messages:
                raise ValueError("no application login replies")
            if not args.skip_ready:
                ready = struct.pack("<HBIH", 5, 1, 2, 13)
                capture.send("ready", frame(9, struct.pack(">H", 1) + ready))
                capture.receive_phase("ready")
                if not any(m["phase"] == "ready" for m in capture.messages):
                    raise ValueError("no application ready replies")
            report["status"] = "passed"
    except Exception as error:
        report["status"] = "failed"
        report["error"] = f"{type(error).__name__}: {error}"
        raise
    finally:
        if proc is not None:
            if proc.poll() is None:
                proc.terminate()
                try:
                    proc.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    proc.kill()
                    proc.wait(timeout=5)
            report["server_stopped"] = proc.poll() is not None
            report["server_exit_code"] = proc.returncode
        if capture is not None:
            report["frames"] = capture.frames
            report["messages"] = capture.messages
            report["received_crc_verified"] = sum(f.get("crc_verified", False) for f in capture.frames)
        (output / "manifest.json").write_text(json.dumps(report, indent=2), encoding="utf-8")
        print(json.dumps({k: report[k] for k in ("capture_directory", "status", "server_stopped")}))


if __name__ == "__main__":
    main()
