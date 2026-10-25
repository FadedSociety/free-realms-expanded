"""Verify a playtest bundle and run a loopback login/create/walk smoke check."""

from __future__ import annotations

import argparse
from contextlib import contextmanager
import json
import math
import os
import secrets
import shutil
import socket
import struct
import subprocess
import sys
import time
import zlib
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from bundle_common import (  # noqa: E402
    BUNDLE_ROOT,
    PROJECT,
    check_activity_content,
    check_runtime_content,
    read_json,
    scan_bundle_sources,
    verify_manifest,
)
from item_content import check_item_content  # noqa: E402


@contextmanager
def scratch_directory():
    # Path.mkdir inherits the workspace ACL here; TemporaryDirectory's 0700
    # Windows ACL is denied by this managed workspace on subsequent access.
    path = BUNDLE_ROOT / f".scratch-check-{secrets.token_hex(8)}"
    path.mkdir()
    try:
        yield path
    finally:
        shutil.rmtree(path)


class Rc4:
    def __init__(self, key: bytes):
        if not key or len(key) > 256:
            raise ValueError("invalid local probe key")
        self.state = list(range(256))
        j = 0
        for i in range(256):
            j = (j + self.state[i] + key[i % len(key)]) & 255
            self.state[i], self.state[j] = self.state[j], self.state[i]
        self.i = 0
        self.j = 0

    def apply(self, data: bytes) -> bytes:
        output = bytearray(data)
        for index in range(len(output)):
            self.i = (self.i + 1) & 255
            self.j = (self.j + self.state[self.i]) & 255
            self.state[self.i], self.state[self.j] = self.state[self.j], self.state[self.i]
            key_byte = self.state[(self.state[self.i] + self.state[self.j]) & 255]
            output[index] ^= key_byte
        return bytes(output)


def seeded_crc32(seed: int, packet: bytes) -> int:
    crc = zlib.crc32(seed.to_bytes(4, "little"))
    return zlib.crc32(packet, crc) & 0xFFFFFFFF


def encode_soe(opcode: int, payload: bytes, seed: int, crc_length: int = 2) -> bytes:
    body = opcode.to_bytes(2, "big") + payload
    checksum = seeded_crc32(seed, body).to_bytes(4, "big")[-crc_length:]
    return body + checksum


def decode_soe(wire: bytes, seed: int, crc_length: int = 2) -> tuple[int, bytes]:
    if len(wire) < 2 + crc_length:
        raise ValueError("truncated local login response")
    body, checksum = wire[:-crc_length], wire[-crc_length:]
    expected = seeded_crc32(seed, body).to_bytes(4, "big")[-crc_length:]
    if checksum != expected:
        raise ValueError("bad checksum in local login response")
    return int.from_bytes(body[:2], "big"), body[2:]


def wire_string(value: str) -> bytes:
    encoded = value.encode("utf-8")
    return len(encoded).to_bytes(4, "little") + encoded


def resource_rows(path: Path):
    for line in path.read_text(encoding="utf-8-sig").splitlines():
        if line and not line.startswith("#"):
            yield line.split("^")


def creator_choices(resources: Path) -> list[int]:
    models = [row for row in resource_rows(resources / "Models.txt") if len(row) >= 14 and row[13] == "1"]
    model, gender = 0, 0
    for row in models:
        try:
            candidate, candidate_gender = int(row[0]), int(row[5])
        except ValueError:
            continue
        if candidate > 0 and candidate_gender in (1, 2):
            model, gender = candidate, candidate_gender
            break
    if not model:
        raise ValueError("no valid player model in bundled login resources")

    mappings = []
    for name in (
        "HeadMappings.txt",
        "HairMappings.txt",
        "FacePaintMappings.txt",
        "SkinToneMappings.txt",
    ):
        ids = [int(row[0]) for row in resource_rows(resources / "CharacterCreate" / name) if row and row[0].isdigit()]
        if not ids:
            raise ValueError(f"no character mapping in {name}")
        mappings.append(ids[0])

    definitions: dict[int, int] = {}
    for row in resource_rows(resources / "CharacterCreate" / "ItemDefinitions.txt"):
        if len(row) >= 42:
            try:
                definition, slot, usage = int(row[0]), int(row[12]), int(row[17])
            except ValueError:
                continue
            if definition > 0 and usage in (0, gender):
                definitions.setdefault(slot, definition)
    clothes = []
    for slot in (6, 5, 4, 3, 1, 2):
        if slot not in definitions:
            raise ValueError(f"no gender-compatible character item for slot {slot}")
        clothes.append(definitions[slot])

    # NativeCreation order: model, four mappings around customization=0,
    # six clothing definitions, six zero tints, eye/hair tint, location.
    return [model, mappings[0], mappings[1], 0, mappings[2], mappings[3], *clothes, *([0] * 9)]


def login_application(handle: str) -> bytes:
    return b"\x01" + wire_string(handle) + wire_string("")


def character_create_application(resources: Path) -> bytes:
    alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
    first = "Probe" + "".join(secrets.choice(alphabet) for _ in range(4))
    last = "Check"
    fields = ["", "", first, last]
    choices = creator_choices(resources)
    if len(choices) != 21:
        raise ValueError("local creator choices have the wrong field count")
    payload = b"".join(wire_string(field) for field in fields)
    payload += b"".join(struct.pack("<i", choice) for choice in choices)
    return b"\x05" + struct.pack("<i", 0) + len(payload).to_bytes(4, "little") + payload


def login_create_character(address: tuple[str, int], key: bytes, handle: str, resources: Path) -> tuple[int, int]:
    """Minimal loopback LoginUdp_6 client for the two native login requests used here."""
    with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as client:
        client.bind(("127.0.0.1", 0))
        client.settimeout(0.5)
        connection_id = int.from_bytes(secrets.token_bytes(4), "big")
        hello = b"\x00\x01" + struct.pack(">III", 2, connection_id, 65000) + b"LoginUdp_6\x00"
        deadline = time.monotonic() + 10
        session = None
        while time.monotonic() < deadline:
            client.sendto(hello, address)
            try:
                response, peer = client.recvfrom(1024)
            except socket.timeout:
                continue
            if peer == address and response[:2] == b"\x00\x02":
                session = response
                break
        if session is None or len(session) != 21:
            raise TimeoutError("local login handshake did not complete")
        if int.from_bytes(session[2:6], "big") != connection_id or session[11:13] != b"\x00\x00":
            raise ValueError("local login handshake response differs")

        seed = int.from_bytes(session[6:10], "big")
        crc_length = session[10]
        if crc_length != 2:
            raise ValueError("local login negotiated an unsupported checksum width")
        outgoing_rc4 = Rc4(key)
        incoming_rc4 = Rc4(key)
        client_sequence = 0

        def exchange(application: bytes) -> bytes:
            nonlocal client_sequence
            ciphertext = outgoing_rc4.apply(application)
            if ciphertext[0] == 0:
                ciphertext = b"\x00" + ciphertext
            payload = client_sequence.to_bytes(2, "big") + ciphertext
            client.sendto(encode_soe(9, payload, seed, crc_length), address)
            client_sequence = (client_sequence + 1) & 0xFFFF
            end = time.monotonic() + 8
            while time.monotonic() < end:
                try:
                    wire, peer = client.recvfrom(65507)
                except socket.timeout:
                    continue
                if peer != address:
                    continue
                opcode, body = decode_soe(wire, seed, crc_length)
                if opcode in (17, 21):
                    continue
                if opcode != 9 or len(body) < 3:
                    continue
                sequence, framed = body[:2], body[2:]
                client.sendto(encode_soe(21, sequence, seed, crc_length), address)
                if framed.startswith(b"\x00\x00"):
                    framed = framed[1:]
                elif framed.startswith(b"\x00\x19"):
                    raise ValueError("unexpected grouped local login response")
                clear = incoming_rc4.apply(framed)
                if clear:
                    return clear
            raise TimeoutError("local login application request timed out")

        reply = exchange(login_application(handle))
        if len(reply) < 2 or reply[0] != 2 or reply[1] != 1:
            raise ValueError("local login service refused the scratch account")
        reply = exchange(character_create_application(resources))
        if len(reply) < 13 or reply[0] != 6:
            raise ValueError("local login service returned no character-create reply")
        result = struct.unpack_from("<i", reply, 1)[0]
        guid = struct.unpack_from("<Q", reply, 5)[0]
        if result != 1 or guid == 0:
            raise ValueError("local login service did not create the scratch character")
        return guid, result


def free_udp_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as sock:
        sock.bind(("127.0.0.1", 0))
        return int(sock.getsockname()[1])


def tail(path: Path) -> str:
    try:
        return "\n".join(path.read_text(encoding="utf-8", errors="replace").splitlines()[-12:])
    except OSError:
        return ""


def wait_for_event(
    process: subprocess.Popen, log: Path, event: str, port: int | None = None
) -> None:
    deadline = time.monotonic() + 15
    needle = f'"event":"{event}"'
    address = f"127.0.0.1:{port}" if port is not None else None
    while time.monotonic() < deadline:
        if process.poll() is not None:
            raise RuntimeError(f"service exited before {event}:\n{tail(log)}")
        try:
            text = log.read_text(encoding="utf-8", errors="replace")
        except OSError:
            text = ""
        if needle in text and (address is None or address in text):
            return
        time.sleep(0.1)
    raise TimeoutError(f"service did not report {event} at {address}:\n{tail(log)}")


def read_event(log: Path, event: str) -> dict:
    for line in log.read_text(encoding="utf-8", errors="replace").splitlines():
        try:
            value = json.loads(line)
        except json.JSONDecodeError:
            continue
        if isinstance(value, dict) and value.get("event") == event:
            return value
    raise ValueError(f"service log has no {event} event")


def stop(process: subprocess.Popen | None, log_stream) -> None:
    if process is not None and process.poll() is None:
        process.terminate()
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=5)
    if log_stream is not None:
        log_stream.close()


BINARIES = ("osfr-login", "osfr-gateway", "osfr-bot")


def existing_binaries(directory: Path) -> dict[str, Path]:
    """The smoke check's binaries in `directory`; an incomplete set is refused."""
    result = {name: directory / f"{name}.exe" for name in BINARIES}
    missing = [name for name, path in result.items() if not path.is_file()]
    if missing:
        raise FileNotFoundError(f"{directory} lacks: {', '.join(missing)}")
    return result


def binaries(runtime: Path) -> dict[str, Path]:
    crate = PROJECT / "prototype" / "osfr-tools"
    command = [
        "powershell.exe",
        "-NoProfile",
        "-ExecutionPolicy",
        "Bypass",
        "-File",
        str(PROJECT / "scripts" / "cargo.ps1"),
        "build",
        "--release",
        "--target-dir",
        str(runtime / "cargo-target"),
        "--manifest-path",
        str(crate / "Cargo.toml"),
        *sum((["--bin", name] for name in BINARIES), []),
    ]
    subprocess.run(command, cwd=PROJECT, check=True, timeout=600)
    return existing_binaries(runtime / "cargo-target" / "release")


def run_smoke(bundle: Path, reference_manifest: dict, exe: dict[str, Path] | None = None) -> None:
    with scratch_directory() as runtime:
        exe = exe or binaries(runtime)
        gateway_port = free_udp_port()
        login_port = free_udp_port()
        while login_port == gateway_port:
            login_port = free_udp_port()
        gateway_address = f"127.0.0.1:{gateway_port}"
        login_address = ("127.0.0.1", login_port)
        identity = runtime / "scratch-identity.json"
        handle = "bundlecheck" + secrets.token_hex(4)
        handoff_key = runtime / "handoff.key"
        handoff_key.write_bytes(secrets.token_bytes(32))
        protocol_key = secrets.token_bytes(16)
        protocol_key_file = runtime / "scratch-client-key.bin"
        protocol_key_file.write_bytes(protocol_key)

        login_log = runtime / "login.log"
        login_stream = login_log.open("wb")
        login_process = subprocess.Popen(
            [
                str(exe["osfr-login"]),
                "--bind",
                f"127.0.0.1:{login_port}",
                "--gateway",
                gateway_address,
                "--resources",
                str(bundle / "login-resources"),
                "--identity",
                str(identity),
                "--state",
                str(runtime / "legacy-state.json"),
                "--key-file",
                str(protocol_key_file),
                "--handoff-key-file",
                str(handoff_key),
                "--session",
                handle,
                "--seconds",
                "90",
            ],
            cwd=PROJECT,
            stdout=login_stream,
            stderr=subprocess.STDOUT,
            creationflags=0x08000000 if os.name == "nt" else 0,
        )
        gateway_process = None
        gateway_stream = None
        bot_result = None
        created_guid = None
        try:
            wait_for_event(login_process, login_log, "listening", login_port)
            created_guid, _ = login_create_character(login_address, protocol_key, handle, bundle / "login-resources")
            created_identity = read_json(identity)
            if created_identity.get("character", {}).get("guid") != created_guid:
                raise ValueError("login identity file does not match the created character")
            stop(login_process, login_stream)
            login_process = None

            initial = reference_manifest["initial_state"]
            if created_guid != initial["guid"]:
                raise ValueError("scratch login GUID differs from the gateway reference GUID")
            gateway_log = runtime / "gateway.log"
            gateway_stream = gateway_log.open("wb")
            gateway_env = os.environ.copy()
            gateway_env["OSFR_GATEWAY_STARTUP_SNAPSHOT"] = str(
                bundle / "startup" / "startup.json"
            )
            gateway_process = subprocess.Popen(
                [
                    str(exe["osfr-gateway"]),
                    gateway_address,
                    "--reference",
                    str(bundle / "reference"),
                    "--state",
                    str(runtime / "gateway-state.json"),
                    "--content",
                    str(bundle / "content"),
                    "--terrain",
                    str(bundle / "terrain"),
                    "--resources",
                    str(bundle / "login-resources"),
                    "--client-locale",
                    str(bundle / "client-locale"),
                    "--seconds",
                    "90",
                ],
                cwd=PROJECT,
                stdout=gateway_stream,
                stderr=subprocess.STDOUT,
                creationflags=0x08000000 if os.name == "nt" else 0,
                env=gateway_env,
            )
            wait_for_event(gateway_process, gateway_log, "listening", gateway_port)

            start = read_json(bundle / "content" / "encounters.json").get("world", {}).get(
                "position"
            )
            if (
                not isinstance(start, list)
                or len(start) != 3
                or any(type(value) not in (int, float) or not math.isfinite(value) for value in start)
            ):
                raise ValueError("gateway world_ready event has no valid character position")
            x, y, z = start
            bot_args = [
                str(exe["osfr-bot"]),
                "--gateway",
                gateway_address,
                "--reference",
                str(bundle / "reference"),
                "--guid",
                str(created_guid),
                "--quiet",
                "--cmd",
                "wait 400",
                "--cmd",
                f"move {x + 0.1:.6f} {y:.6f} {z:.6f}",
                "--cmd",
                "wait 400",
                "--cmd",
                f"move {x + 0.2:.6f} {y:.6f} {z:.6f}",
                "--cmd",
                "wait 200",
            ]
            bot_result = subprocess.run(
                bot_args,
                cwd=PROJECT,
                capture_output=True,
                text=True,
                timeout=25,
            )
            if bot_result.returncode != 0:
                raise RuntimeError(
                    "osfr-bot failed:\n" + "\n".join(bot_result.stdout.splitlines()[-10:])
                )
            ready = read_event(gateway_log, "world_ready").get("state", {}).get("position")
            if not isinstance(ready, list) or len(ready) != 3 or any(
                abs(float(actual) - expected) > 0.01
                for actual, expected in zip(ready, start)
            ):
                raise ValueError("gateway world_ready position differs from bundled encounter start")
            # This scratch run deliberately uses osfr-login's legacy local
            # session, so the gateway persists the character at --state.
            save_path = runtime / "gateway-state.json"
            deadline = time.monotonic() + 5
            while not save_path.is_file() and time.monotonic() < deadline:
                time.sleep(0.1)
            if not save_path.is_file():
                raise RuntimeError("gateway did not persist the scratch character")
            snapshot = read_json(save_path)
            saved = snapshot.get("character", {})
            position = saved.get("position")
            if saved.get("guid") != created_guid or not isinstance(position, list) or len(position) != 3:
                raise ValueError("gateway scratch character save is invalid")
            walked = ((position[0] - x) ** 2 + (position[1] - y) ** 2 + (position[2] - z) ** 2) ** 0.5
            if not 0.05 <= walked <= 1.0:
                raise RuntimeError(f"osfr-bot walk was not persisted (distance={walked:.3f} m)")
            stop(gateway_process, gateway_stream)
            gateway_process = None
            gateway_stream = None
            print(f"Login service: PASS (127.0.0.1:{login_port}; fresh character created)")
            print(f"Gateway: PASS (127.0.0.1:{gateway_port}; single local scratch session)")
            print(f"osfr-bot: PASS (gateway login and persisted walk {walked:.3f} m)")
        except Exception:
            for label, log in (("login", login_log), ("gateway", runtime / "gateway.log")):
                lines = tail(log)
                if lines:
                    print(f"{label} service log tail:\n{lines}", file=sys.stderr)
            raise
        finally:
            stop(login_process, login_stream if login_process is not None else None)
            stop(gateway_process, gateway_stream)


def latest_bundle() -> Path:
    candidates = sorted(
        (path for path in BUNDLE_ROOT.glob("playtest-*") if path.is_dir()),
        key=lambda path: path.name,
    )
    if not candidates:
        raise FileNotFoundError("no lab/bundles/playtest-* folder exists")
    return candidates[-1]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bundle", type=Path, help="bundle directory (default: newest playtest bundle)")
    parser.add_argument("--verify-only", action="store_true", help="verify manifest and source exclusions; do not start services")
    parser.add_argument("--bin-dir", type=Path, help="reuse this release build (osfr-login, osfr-gateway, osfr-bot) instead of building one")
    args = parser.parse_args()
    exe = existing_binaries(args.bin_dir.resolve(strict=True)) if args.bin_dir else None
    if exe:
        print(f"Binaries: PASS (reusing {', '.join(BINARIES)} from {args.bin_dir})")
    bundle = (args.bundle or latest_bundle()).resolve(strict=True)
    manifest, file_count = verify_manifest(bundle)
    protected_count, _ = scan_bundle_sources(bundle, manifest)
    print(f"Manifest: PASS ({file_count} payload files, SHA-256 and sizes match)")
    print(f"Frozen activity content: PASS ({check_activity_content(bundle)})")
    print(f"Runtime content: PASS (optional files present and valid: {check_runtime_content(bundle) or 'none'})")
    print(f"Item content: PASS ({check_item_content(bundle)})")
    print(
        "Security scan: PASS (empty accounts=0, characters=0, public keys=0; "
        f"protected source files checked={protected_count}, matches=0)"
    )
    if args.verify_only:
        return 0
    reference_manifest = read_json(bundle / "reference" / "manifest.json")
    run_smoke(bundle, reference_manifest, exe)
    print("Bundle check: PASS")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as error:
        print(f"check_bundle: {error}", file=sys.stderr)
        raise SystemExit(1)
