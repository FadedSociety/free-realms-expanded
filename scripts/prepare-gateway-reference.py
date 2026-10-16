"""Prepare private Rust gateway templates from the verified original-client trace.

Game-derived bytes are written only beneath the sibling analysis lab. Existing
outputs are never overwritten. No client, legacy server, or network is started.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import struct


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def checked_file(root: Path, relative: str, expected: str) -> bytes:
    path = (root / relative).resolve()
    if not path.is_relative_to(root.resolve()) or not path.is_file():
        raise ValueError("capture path escapes its source directory")
    data = path.read_bytes()
    if digest(data) != expected:
        raise ValueError(f"capture SHA-256 mismatch: {relative}")
    return data


def tunnel(data: bytes) -> tuple[int, bytes]:
    if len(data) < 9 or data[:3] != b"\x05\x00\x01":
        raise ValueError("expected a reliable client tunnel")
    if int.from_bytes(data[3:7], "little") != len(data) - 7:
        raise ValueError("tunnel length mismatch")
    return int.from_bytes(data[7:9], "little"), data[9:]


def string_end(data: bytes, offset: int) -> int:
    if offset + 4 > len(data):
        raise ValueError("truncated string length")
    size = int.from_bytes(data[offset:offset + 4], "little")
    end = offset + 4 + size
    if size > 4096 or end > len(data):
        raise ValueError("invalid prefix string")
    data[offset + 4:end].decode("utf-8")
    return end


def self_layout(data: bytes) -> dict:
    opcode, payload = tunnel(data)
    if opcode != 12 or len(payload) < 4:
        raise ValueError("expected PacketSendSelfToClient")
    if int.from_bytes(payload[:4], "little") != len(payload) - 4:
        raise ValueError("self payload length mismatch")
    pc = payload[4:]
    # Recovered ClientPcData.ReadFromJSON: unknown i64, GUID i64, model i32,
    # head/hair strings, hair/eye colors i32, three appearance strings,
    # five unknown i32, position[4], rotation[4]. These are not searched bytes.
    offset = 20
    offset = string_end(pc, offset)
    offset = string_end(pc, offset)
    offset += 8
    for _ in range(3):
        offset = string_end(pc, offset)
    offset += 20
    if offset + 32 > len(pc):
        raise ValueError("truncated self position/rotation")
    return {"guid_offset": 13 + 8, "position_offset": 13 + offset,
            "rotation_offset": 13 + offset + 16}


def zone_layout(data: bytes) -> dict:
    opcode, payload = tunnel(data)
    if opcode != 43:
        raise ValueError("expected PacketSendZoneDetails")
    # Sanctuary PacketSendZoneDetails: name, type i32, two booleans, sky,
    # is-in-arena boolean, Id i32, GeometryId i32, two booleans.
    offset = string_end(payload, 0) + 6
    offset = string_end(payload, offset) + 1
    if offset + 10 != len(payload):
        raise ValueError("unexpected zone-details layout")
    return {"zone_id_offset": 9 + offset}


def main() -> None:
    project = Path(__file__).resolve().parents[1]
    lab = project.parent / "lab"
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--capture", type=Path,
                        default=lab / "runs" / "client-baseline-20261008T020205Z")
    parser.add_argument("--output", type=Path,
                        default=lab / "references" / "rust-gateway-reference")
    args = parser.parse_args()
    capture = args.capture.resolve()
    output = args.output.resolve()
    if not output.is_relative_to(lab.resolve()):
        raise ValueError("private templates must stay inside the analysis lab")
    analysis_path = capture / "analysis" / "packet-analysis.json"
    capture_bytes = (capture / "manifest.json").read_bytes()
    analysis_bytes = analysis_path.read_bytes()
    manifest = json.loads(capture_bytes)
    analysis = json.loads(analysis_bytes)
    for frame in manifest["frames"]:
        checked_file(capture, frame["path"], frame["sha256"])
    server = analysis["directions"]["server-to-client"]
    if server["sequence_gaps"] or server["incomplete_fragment"]:
        raise ValueError("reference capture has an incomplete reliable stream")
    source_messages = server["messages"][:46]
    if len(source_messages) != 46:
        raise ValueError("reference capture lacks the 46 initial replies")
    templates = [checked_file(capture, m["path"], m["sha256"])
                 for m in source_messages]
    opcodes = [tunnel(data)[0] for data in templates]
    if opcodes[:8] != [41, 2, 166, 43, 144, 35, 193, 12]:
        raise ValueError("unexpected reference login sequence")
    if opcodes[8] != 67 or 14 not in opcodes[8:]:
        raise ValueError("unexpected reference ready sequence")
    sl = self_layout(templates[7])
    zl = zone_layout(templates[3])
    pc_data_path = lab / "working" / "OSFR.Server" / "OSFR Server" / "Customize" / "PacketSendSelfToClient.json"
    pc_bytes = pc_data_path.read_bytes()
    pc = json.loads(pc_bytes)
    initial = {
        "guid": int.from_bytes(templates[7][sl["guid_offset"]:sl["guid_offset"] + 8], "little"),
        "zone_id": int.from_bytes(templates[3][zl["zone_id_offset"]:zl["zone_id_offset"] + 4], "little"),
        "position": list(struct.unpack_from("<3f", templates[7], sl["position_offset"])),
        "rotation": list(struct.unpack_from("<3f", templates[7], sl["rotation_offset"])),
    }
    if initial["guid"] != pc["PlayerGUID"]:
        raise ValueError("self GUID differs from recovered character data")
    for field, key in (("position_offset", "PlayerPosition"), ("rotation_offset", "CameraRotation")):
        expected = struct.pack("<4f", *pc[key])
        if templates[7][sl[field]:sl[field] + 16] != expected:
            raise ValueError(f"self {key} differs from proven serialized fields")
    # The actual client packet layout agrees with Sanctuary PacketLogin.
    login_record = next(m for m in analysis["directions"]["client-to-server"]["messages"]
                        if m["outer_opcode"] == 1)
    login = checked_file(capture, login_record["path"], login_record["sha256"])
    offset = string_end(login, 2) + 8
    version_end = string_end(login, offset)
    version = login[offset + 4:version_end].decode("utf-8")
    if string_end(login, version_end) != len(login):
        raise ValueError("login fields do not exactly consume the reference")
    output.mkdir(parents=True, exist_ok=False)
    entries = []
    for index, (data, source) in enumerate(zip(templates, source_messages)):
        kind = "login" if index < 8 else "ready"
        name = f"{index:04d}-{kind}.bin"
        (output / name).write_bytes(data)
        entries.append({"kind": kind, "file": name, "length": len(data),
                        "sha256": digest(data), "opcode": opcodes[index]})
    result = {"schema_version": 1, "client_version": version,
              "initial_state": initial, "self_layout": sl, "zone_layout": zl,
              "templates": entries,
              "provenance": {"capture_directory": str(capture),
                             "capture_manifest_sha256": digest(capture_bytes),
                             "packet_analysis_sha256": digest(analysis_bytes),
                             "character_json_sha256": digest(pc_bytes),
                             "verified_source_frames": len(manifest["frames"]),
                             "self_layout_source": "recovered ClientPcData.ReadFromJSON prefix",
                             "zone_layout_source": "pinned Sanctuary PacketSendZoneDetails"}}
    (output / "manifest.json").write_text(json.dumps(result, indent=2), encoding="utf-8")
    print(json.dumps({"reference_directory": str(output), "templates": len(entries),
                      "self_layout": sl, "zone_layout": zl}))


if __name__ == "__main__":
    main()
