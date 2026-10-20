#!/usr/bin/env python3
"""Run the W107 scratch-gateway playtest rehearsal; all runtime data stays in lab."""

from __future__ import annotations

import argparse
import csv
import ctypes
import datetime as dt
import io
import hashlib
import json
import math
import os
import random
import re
import shutil
import socket
import struct
import subprocess
import sys
import tarfile
import threading
import time
from pathlib import Path


BOT_LAST_NAME = "Test"
CLIENT_VERSION = "1.910.1.530630"
MAX_EXPECT_MS = 15000


def login_summary(run_dir: Path) -> dict:
    if not list(run_dir.glob("*.profile.json")):
        return {"broker_refusals": 0, "login_failures": 0, "native_logins": 0}
    # Admission refusals are intentionally silent on the server. Count fixed
    # client messages too, without treating an absent server event as success.
    outputs = []
    for path in (run_dir / "bots").glob("bot-*.stdout.log"):
        outputs.append("\n".join(read_lines(path) + read_lines(path.with_name(path.name.replace(".stdout.", ".stderr.")))))
    outputs.extend("\n".join(read_lines(p)) for p in (run_dir / "bots").glob("save-load-*.log"))
    probe = run_dir / "bots" / "duplicate-session-probe.log"
    if probe.is_file():
        outputs.append("\n".join(read_lines(probe)))
    server_events = sum('"event":"auth_refused"' in line
        for name in ("login.stdout.log", "login.stderr.log") for line in read_lines(run_dir / name))
    return {"broker_refusals": max(server_events, sum("broker refused" in text for text in outputs)),
        "login_failures": sum("native login selected character" not in text for text in outputs),
        "native_logins": sum("native login selected character" in text for text in outputs)}


def scan_private_keys(run_dir: Path, records: list[dict]) -> dict:
    import base64
    secrets = []
    for record in records:
        value = record.get("private_seed")
        if value:
            secrets.extend([value, value.hex().encode(), base64.b64encode(value)])
    matches = []
    logs = [p for p in run_dir.rglob("*") if p.is_file() and p.suffix in (".log", ".jsonl")]
    for path in logs:
        data = path.read_bytes()
        if any(secret in data for secret in secrets) or re.search(rb'(?i)(?<![0-9a-f])[0-9a-f]{32}(?![0-9a-f])', data):
            matches.append(str(path.relative_to(run_dir)))
    return {"files_scanned": len(logs), "private_keys_checked": len(secrets) // 3, "matches": matches}


def run_bot(args: list[str], record: dict, run_dir: Path, timeout: float):
    # Private seeds never enter argv, environment variables or persisted files.
    result = subprocess.run(args, cwd=run_dir, input=record.get("private_seed"), capture_output=True, timeout=timeout)
    return subprocess.CompletedProcess(args, result.returncode,
        result.stdout.decode("utf-8", errors="replace"), result.stderr.decode("utf-8", errors="replace"))


def now_utc() -> dt.datetime:
    return dt.datetime.now(dt.timezone.utc)


def utc_stamp() -> str:
    return now_utc().strftime("%Y%m%dT%H%M%SZ")


def put_string(value: str) -> bytes:
    raw = value.encode("utf-8")
    return struct.pack("<I", len(raw)) + raw


def npc_interact(npc_id: int) -> bytes:
    return bytes((26, 0, 8, 0)) + struct.pack("<Q", npc_id) + b"\0"


def npc_select(npc_id: int, choice: int) -> bytes:
    return bytes((26, 0, 10, 0)) + struct.pack("<Qi", npc_id, choice)


def quest_dialogue_response(choice: int = 301) -> bytes:
    return bytes((26, 0, 6, 0)) + struct.pack("<i", choice)


def quest_reply(quest_id: int, accept: bool = True) -> bytes:
    return bytes((49, 0, 2, 0, 0, 0)) + struct.pack("<I", quest_id) + bytes((int(accept),))


def shop_buy(vendor_id: int, item_id: int, transaction_id: int) -> bytes:
    return bytes((165, 0, 4, 0)) + struct.pack("<QI iII", vendor_id, transaction_id, item_id, 1, 0)


def friend_request(first: str, last: str) -> bytes:
    return bytes((26, 0, 14, 0)) + put_string(first) + put_string(last)


def social_name(first: str, last: str) -> bytes:
    return struct.pack("<iii", 0, 0, 0) + put_string(first) + put_string(last)


def group_invite(sender_guid: int, sender: tuple[str, str], target: tuple[str, str]) -> bytes:
    # Client writer 00c235c0: family/sub header, then 00a95c50 writes both
    # guid/NameData records. The server resolves the target by name, so its guid is a placeholder.
    return (
        bytes((40, 0, 1, 0))
        + struct.pack("<Q", sender_guid)
        + social_name(*sender)
        + struct.pack("<Q", 0)
        + social_name(*target)
        + struct.pack("<I5I", 0, 0, 0, 0, 0, 0)
    )


def tell_packet(sender_guid: int, sender: tuple[str, str], target_guid: int, target: tuple[str, str], text: str) -> bytes:
    out = bytearray((15, 0, 1, 0))
    out.extend(struct.pack("<HQQ", 1, sender_guid, target_guid))
    out.extend(social_name(*sender))
    out.extend(social_name(*target))
    out.extend(struct.pack("<i", len(text.encode("utf-8"))))
    out.extend(text.encode("utf-8"))
    out.extend(struct.pack("<4fQi", 0.0, 0.0, 0.0, 0.0, 0, 0))
    return bytes(out)


def raw_step(packet: bytes) -> str:
    return "raw " + packet.hex()


def load_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, value) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")


def free_port(protocol: str) -> int:
    kind = socket.SOCK_DGRAM if protocol == "udp" else socket.SOCK_STREAM
    with socket.socket(socket.AF_INET, kind) as sock:
        sock.bind(("127.0.0.1", 0))
        return sock.getsockname()[1]


def run_command(args: list[str], cwd: Path, env: dict[str, str] | None = None, timeout: int = 600):
    return subprocess.run(args, cwd=cwd, env=env, text=True, encoding="utf-8", errors="replace", capture_output=True, timeout=timeout)


def stage_head(root: Path, build_dir: Path, commit: str) -> Path:
    source = build_dir / "head-src"
    if source.resolve().parent != build_dir.resolve() or source.is_symlink():
        raise RuntimeError(f"unsafe HEAD staging path: {source}")
    marker = source / ".w107-head"
    if marker.is_file() and marker.read_text(encoding="ascii").strip() == commit:
        manifest = source / "prototype" / "osfr-tools" / "Cargo.toml"
        if manifest.is_file():
            return manifest
    if source.exists():
        shutil.rmtree(source)
    source.mkdir(parents=True)
    archive = subprocess.run(["git", "archive", "--format=tar", commit, "prototype/osfr-tools"], cwd=root, capture_output=True, check=True).stdout
    with tarfile.open(fileobj=io.BytesIO(archive), mode="r:") as tar:
        tar.extractall(source, filter="data")
    marker.write_text(commit + "\n", encoding="ascii")
    manifest = source / "prototype" / "osfr-tools" / "Cargo.toml"
    if not manifest.is_file():
        raise RuntimeError("HEAD archive lacks prototype/osfr-tools/Cargo.toml")
    return manifest


def build_bins(root: Path, build_dir: Path, run_dir: Path, commit: str, working_tree: bool = False) -> Path:
    manifest = stage_head(root, build_dir, commit)
    if working_tree:
        # Freeze current sources so an uncommitted fix is actually exercised.
        crate = root / "prototype" / "osfr-tools"
        for name in ("src", "tests", "data"):
            shutil.copytree(crate / name, manifest.parent / name, dirs_exist_ok=True)
        for name in ("Cargo.toml", "Cargo.lock"):
            shutil.copy2(crate / name, manifest.parent / name)
        (run_dir / "build.diff.patch").write_text(
            run_command(["git", "diff", "HEAD", "--", "prototype/osfr-tools"], root).stdout,
            encoding="utf-8",
        )
        # Never reuse this overlaid snapshot as a clean HEAD on a later run.
        (manifest.parents[2] / ".w107-head").unlink(missing_ok=True)
    (run_dir / "build.sources.json").write_text(json.dumps({
        str(path.relative_to(manifest.parent)): hashlib.sha256(path.read_bytes()).hexdigest()
        for path in sorted(manifest.parent.rglob("*")) if path.is_file()
    }, indent=2) + "\n", encoding="utf-8")
    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = str(build_dir)
    command = [
        "powershell.exe", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File",
        str(root / "scripts" / "cargo.ps1"), "build", "--release", "--manifest-path", str(manifest),
        "--bin", "osfr-gateway", "--bin", "osfr-login", "--bin", "osfr-bot", "--bin", "osfr-account",
    ]
    result = run_command(command, root, env=env, timeout=900)
    (run_dir / "build.stdout.log").write_text(result.stdout, encoding="utf-8")
    (run_dir / "build.stderr.log").write_text(result.stderr, encoding="utf-8")
    if result.returncode:
        raise RuntimeError(f"cargo build failed ({result.returncode}); see build logs")
    suffix = ".exe" if os.name == "nt" else ""
    return build_dir / "release" / ("osfr-gateway" + suffix)


def copy_runtime_content(base_content: Path, root: Path, run_dir: Path) -> Path:
    content = run_dir / "content"
    if not base_content.is_dir():
        raise RuntimeError(f"HEAD content directory missing: {base_content}")
    shutil.copytree(base_content, content)
    bundle = root.parent / "lab" / "reports" / "round70-native-20261016T061000Z" / "content"
    if not bundle.is_dir():
        raise RuntimeError(f"required content bundle not found: {bundle}")
    shutil.copytree(bundle, content, dirs_exist_ok=True)

    # (R) Scratch-only wallet seed lets each test character buy one low-cost item.
    # Keep the local span digest aligned; no modified value is presented as recovered.
    inventory_path = content / "inventory.json"
    inventory = load_json(inventory_path)
    for key in ("default_state", "initial_state"):
        state = inventory.get(key)
        if isinstance(state, dict):
            state["coins"] = max(5000, int(state.get("coins", 0)))
    coin_bytes = struct.pack("<I", int(inventory["default_state"]["coins"]))
    for span in inventory.get("spans", []):
        if span.get("kind") == "coins":
            span["sha256"] = hashlib.sha256(coin_bytes).hexdigest()
    write_json(inventory_path, inventory)
    return content


def scenario_from_content(content: Path) -> dict:
    npc_rows = []
    for path in sorted((content / "npcs").glob("*.json")):
        npc_rows.extend(load_json(path).get("records", []))
    npcs = {int(row["npc_id"]): row for row in npc_rows}
    prices = {int(row["item_id"]): row.get("purchase_price_coins") for row in load_json(content / "stores.json")["items"]}
    item_doc = load_json(content / "item-catalogue.json")
    catalog_rows = item_doc.get("items", item_doc.get("records", item_doc.get("definitions", [])))
    catalog_ids = set()
    for item in catalog_rows:
        for key in ("item_id", "definition_id", "id"):
            if isinstance(item.get(key), int):
                catalog_ids.add(item[key])
                break
    encounter_file = content / "encounters.json"
    start_position = load_json(encounter_file)["world"]["position"] if (content / "opening.json").is_file() and encounter_file.is_file() else None
    quest_candidates = []
    for path in sorted((content / "quests").glob("*.json")):
        for candidate in load_json(path).get("records", []):
            giver = npcs.get(int(candidate.get("giver_npc_id", -1)))
            if (candidate.get("enabled", True) and not candidate.get("prerequisite_quest_ids")
                    and giver and giver.get("position")
                    and int(candidate["id"]) in giver.get("quest_ids_given", [])
                    and not candidate.get("job")):  # job-gated quests need that active profile
                distance = math.dist(start_position, giver["position"]) if start_position else 0.0
                quest_candidates.append((distance, int(candidate["id"]), candidate, giver))
    if not quest_candidates:
        raise RuntimeError("no enabled quest without prerequisites has a positioned, matching giver")
    _, _, quest, quest_npc = min(quest_candidates, key=lambda row: (row[0], row[1]))
    quest_npc = npcs.get(int(quest["giver_npc_id"]))
    if not quest_npc or not quest_npc.get("position"):
        raise RuntimeError(f"quest giver for {quest['id']} has no positioned NPC record")

    vendors = load_json(content / "vendors.json").get("vendors", [])
    vendor_choices = []
    qpos = quest_npc["position"]
    for vendor in vendors:
        npc = npcs.get(int(vendor["npc_id"]))
        if not npc or not npc.get("position") or "vendor" not in npc.get("roles", []):
            continue
        for item_id in vendor.get("item_ids", []):
            price = prices.get(int(item_id))
            if price is None or int(price) > 1000 or catalog_ids and int(item_id) not in catalog_ids:
                continue
            distance = math.dist(qpos, npc["position"])
            vendor_choices.append((distance, int(price), int(npc["npc_id"]), int(item_id), npc))
    if not vendor_choices:
        raise RuntimeError("no positioned vendor item under 1000 coins matched the inventory catalogue")
    _, price, vendor_id, item_id, vendor_npc = min(vendor_choices)
    if start_position is None:
        start_position = quest_npc["position"]
    enemies_doc = load_json(content / "field_enemies.json").get("enemies", [])
    combat_quest_id = 3030007
    enemies = [
        enemy for enemy in enemies_doc
        if combat_quest_id in enemy.get("quest_ids", []) and len(enemy.get("position", [])) == 3
    ]
    if not enemies:
        raise RuntimeError("quest 3030007 has no field-enemy rows in scratch content")
    enemies.sort(key=lambda row: int(row["guid"]))
    return {
        "quest_id": int(quest["id"]),
        "combat_quest_id": combat_quest_id,
        "quest_npc_id": int(quest_npc["npc_id"]),
        "quest_npc_position": [float(x) for x in quest_npc["position"]],
        "start_position": [float(x) for x in start_position],
        "vendor_id": vendor_id,
        "vendor_position": [float(x) for x in vendor_npc["position"]],
        "item_id": item_id,
        "item_price": price,
        "enemies": [{"guid": int(row["guid"]), "position": [float(x) for x in row["position"]]} for row in enemies[:8]],
        "npc_count": len(npcs),
    }


def write_login_resources(run_dir: Path) -> None:
    resources = run_dir / "login-resources"
    creator = resources / "CharacterCreate"
    creator.mkdir(parents=True)
    model = ["0"] * 17
    model[0] = model[5] = model[13] = "1"
    (resources / "Models.txt").write_text("^".join(model), encoding="utf-8")
    for filename, row in [
        ("HeadMappings.txt", "1^head.adr^"),
        ("HairMappings.txt", "2^hair.adr^"),
        ("ModelCustomizationMappings.txt", "1^beard.adr^"),
        ("FacePaintMappings.txt", "0^none^"),
        ("SkinToneMappings.txt", "1^skin.adr^"),
    ]:
        (creator / filename).write_text(row, encoding="utf-8")
    rows = []
    for item_id, slot in ((201, 5), (189, 4), (199, 3)):
        row = ["0"] * 42
        row[0] = str(item_id)
        row[12] = str(slot)
        row[16:20] = ["item.adr", "1", "cloth", "0"]
        row[41] = "tint"
        rows.append("^".join(row))
    (creator / "ItemDefinitions.txt").write_text("\n".join(rows) + "\n", encoding="utf-8")


def create_accounts(account_exe: Path, run_dir: Path, scenario: dict, bots: int, broker_login: bool = False) -> tuple[Path, list[dict]]:
    from cryptography.hazmat.primitives import serialization
    from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey

    store_path = run_dir / "accounts.json"
    records = []
    for index in range(bots):
        handle = f"stressbot{index + 1:02d}"
        result = None
        for _ in range(8):
            private = Ed25519PrivateKey.generate()
            public = private.public_key().public_bytes(serialization.Encoding.Raw, serialization.PublicFormat.Raw)
            del private
            result = run_command([
                str(account_exe), "create", "--store", str(store_path), "--handle", handle,
                "--public-key", public.hex(),
            ], run_dir)
            if result.returncode == 0:
                break
        if result is None or result.returncode:
            raise RuntimeError(f"osfr-account failed to create scratch account {index + 1}: {result.stderr.strip() if result else 'no result'}")
        if broker_login:
            private = Ed25519PrivateKey.generate()
            login_public = private.public_key().public_bytes(serialization.Encoding.Raw, serialization.PublicFormat.Raw)
            seed = private.private_bytes(serialization.Encoding.Raw, serialization.PrivateFormat.Raw, serialization.NoEncryption())
            del private
            # Exercise operator enrolment explicitly; remove the discarded bootstrap key.
            for operation, enrolled in (("add-key", login_public), ("remove-key", public)):
                result = run_command([str(account_exe), operation, "--store", str(store_path),
                    "--account-id", str(index + 1), "--public-key", enrolled.hex()], run_dir)
                if result.returncode:
                    raise RuntimeError(f"osfr-account {operation} failed for scratch account {index + 1}")
        records.append({"handle": handle, "account_id": index + 1, "guid": 1001 + index, "first": f"Test{chr(ord('A') + index)}", "last": BOT_LAST_NAME, **({"private_seed": seed} if broker_login else {})})

    document = load_json(store_path)
    store = document["store"]
    for index, record in enumerate(records):
        guid = record["guid"]
        pos = scenario["start_position"]
        character = {
            "guid": guid,
            "owner_account_id": record["account_id"],
            "identity": {
                "guid": 1,
                "first_name": record["first"],
                "last_name": record["last"],
                "model": 1,
                "head": "head.adr",
                "hair": "hair.adr",
                "model_customization": "beard.adr",
                "face_paint": "none",
                "skin_tone": "skin.adr",
                "eye_color": 1,
                "hair_color": 1,
                "gender": 1,
                "creation": {
                    "first_name": record["first"],
                    "last_name": record["last"],
                    "temporary_first_name": "Temp" + chr(65 + index),
                    "temporary_last_name": "Name" + chr(65 + index),
                    "choices": [0] * 21,
                },
                "attachments": [],
            },
            "world_state": {"guid": guid, "zone_id": 0, "position": pos, "rotation": [0.0, 0.0, 1.0]},
        }
        store["accounts"][str(record["account_id"])]["character_guids"] = [guid]
        store["characters"][str(guid)] = character
    store["next_character_guid"] = 1001 + bots
    write_json(store_path, document)
    return store_path, records


class ProcessLogs:
    def __init__(self, name: str, executable: Path, args: list[str], cwd: Path, log_dir: Path):
        self.name = name
        self.process = subprocess.Popen(
            [str(executable), *args], cwd=cwd, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
            text=True, encoding="utf-8", errors="replace", bufsize=1,
        )
        self._files = [
            (log_dir / f"{name}.stdout.log").open("a", encoding="utf-8", buffering=1),
            (log_dir / f"{name}.stderr.log").open("a", encoding="utf-8", buffering=1),
        ]
        self._threads = []
        for stream, destination in zip((self.process.stdout, self.process.stderr), self._files):
            thread = threading.Thread(target=self._copy, args=(stream, destination), daemon=True)
            thread.start()
            self._threads.append(thread)

    @staticmethod
    def _copy(stream, destination):
        try:
            for line in iter(stream.readline, ""):
                destination.write(line)
        finally:
            destination.flush()

    def close_logs(self):
        for thread in self._threads:
            thread.join(timeout=2)
        for handle in self._files:
            handle.close()


def wait_ready(process: ProcessLogs, path: Path, tokens: tuple[str, ...], timeout: float = 30.0) -> None:
    deadline = time.time() + timeout
    stdout_path = path / f"{process.name}.stdout.log"
    while time.time() < deadline:
        if process.process.poll() is not None:
            raise RuntimeError(f"{process.name} exited before ready with {process.process.returncode}")
        text = stdout_path.read_text(encoding="utf-8", errors="replace") if stdout_path.exists() else ""
        if all(token in text for token in tokens):
            return
        time.sleep(0.1)
    raise TimeoutError(f"{process.name} did not emit ready markers {tokens}")


def gateway_args(bind_port: int, reference: Path, content: Path, state: Path, store: Path, handoff: Path, run_dir: Path, trace_name: str | None = None) -> list[str]:
    return [
        f"127.0.0.1:{bind_port}", "--reference", str(reference), "--state", str(state),
        "--content", str(content), "--account-store", str(store), "--handoff-key-file", str(handoff),
        "--studio-port", "0", "--security-log", str(run_dir / "security.jsonl"),
    ] + (["--trace", str(run_dir / trace_name)] if trace_name else [])


def write_raw(lines: list[str], packet: bytes) -> None:
    lines.append(raw_step(packet))
    lines.append("wait 350")


def move_steps(current: list[float], target: list[float], lines: list[str], rng: random.Random) -> list[float]:
    distance = math.dist(current, target)
    steps = max(1, math.ceil(distance / 12.0))
    start = current[:]
    for step in range(1, steps + 1):
        p = [start[i] + (target[i] - start[i]) * step / steps for i in range(3)]
        # (R) 12 m/400 ms is within the local reconstructed movement soft-speed limit.
        lines.append("move " + " ".join(f"{value:.3f}" for value in p))
        lines.append(f"wait {rng.randint(380, 460)}")
    return target[:]


def make_bot_script(index: int, record: dict, records: list[dict], scenario: dict, seconds: int, initial: bool, start_position: list[float], output: Path) -> list[float]:
    rng = random.Random((index + 1) * 7919 + int(time.time()))
    lines: list[str] = []
    current = start_position[:]
    partner = records[(index + 1) % len(records)]
    tell_target = records[(index - 1) % len(records)]
    cycle = 0

    def ping() -> None:
        nonlocal cycle
        token = f"stressping_{record['guid']}_{cycle}"
        lines.extend([f"chat {token}", f"expect-text {token}"])
        cycle += 1

    if initial:
        current = move_steps(current, scenario["quest_npc_position"], lines, rng)
        ping()
        write_raw(lines, npc_interact(scenario["quest_npc_id"]))
        write_raw(lines, npc_select(scenario["quest_npc_id"], 1))
        write_raw(lines, quest_dialogue_response())
        write_raw(lines, quest_reply(scenario["quest_id"], True))

        enemy = scenario["enemies"][index % len(scenario["enemies"])]
        current = move_steps(current, enemy["position"], lines, rng)
        lines.extend([f"ability 0 {enemy['guid']}", "wait 800"])
        current = move_steps(current, scenario["vendor_position"], lines, rng)
        write_raw(lines, npc_interact(scenario["vendor_id"]))
        write_raw(lines, npc_select(scenario["vendor_id"], 2))
        lines.append("wait 800")
        write_raw(lines, shop_buy(scenario["vendor_id"], scenario["item_id"], record["guid"] * 100 + 1))
        write_raw(lines, friend_request(partner["first"], partner["last"]))
        write_raw(lines, group_invite(record["guid"], (record["first"], record["last"]), (partner["first"], partner["last"])))
        write_raw(lines, tell_packet(record["guid"], (record["first"], record["last"]), tell_target["guid"], (tell_target["first"], tell_target["last"]), f"stress_tell_{record['guid']}"))

    # Repeated randomized play keeps each bot connected for its phase; each phase
    # ends normally so the next session tests persisted state after reconnect.
    estimated = sum(0.08 for line in lines if not line.startswith("wait ")) + sum(int(line.split()[1]) / 1000 for line in lines if line.startswith("wait "))
    while estimated < seconds - 22:
        targets = [scenario["quest_npc_position"], scenario["vendor_position"], scenario["enemies"][rng.randrange(len(scenario["enemies"]))]["position"]]
        target = targets[(cycle + index + rng.randrange(len(targets))) % len(targets)]
        current = move_steps(current, target, lines, rng)
        if target in [enemy["position"] for enemy in scenario["enemies"]]:
            enemy = min(scenario["enemies"], key=lambda item: math.dist(item["position"], target))
            lines.extend([f"ability 0 {enemy['guid']}", f"wait {rng.randint(350, 900)}"])
        ping()
        if cycle % 3 == 0:
            write_raw(lines, tell_packet(record["guid"], (record["first"], record["last"]), tell_target["guid"], (tell_target["first"], tell_target["last"]), f"stress_tell_{record['guid']}_{cycle}"))
        pause = rng.randint(12, 28)
        lines.append(f"wait {pause * 1000}")
        estimated = sum(0.08 for line in lines if not line.startswith("wait ")) + sum(int(line.split()[1]) / 1000 for line in lines if line.startswith("wait "))
    remaining_ms = max(500, int((seconds - estimated) * 1000))
    lines.extend([f"wait {remaining_ms}", "quit"])
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return current


def bot_args(bot_exe: Path, gateway_port: int, store: Path, handoff: Path, record: dict, name: str, script: Path) -> list[str]:
    if "broker_profile" in record:
        return [str(bot_exe), "--broker-profile", str(record["broker_profile"]), "--guid", str(record["guid"]),
            "--name", name, "--version", CLIENT_VERSION, "--script", str(script), "--expect-timeout", str(MAX_EXPECT_MS)]
    return [
        str(bot_exe), "--gateway", f"127.0.0.1:{gateway_port}", "--account-store", str(store),
        "--handoff-key-file", str(handoff), "--guid", str(record["guid"]), "--name", name,
        "--version", CLIENT_VERSION, "--script", str(script), "--expect-timeout", str(MAX_EXPECT_MS),
    ]


def bot_worker(index: int, state: dict, stop: threading.Event, registry: dict, registry_lock: threading.Lock, timeline, duration: int, gateway_port: int, store: Path, handoff: Path, records: list[dict], scenario: dict, bin_dir: Path, run_dir: Path, start_time: float, chaos_time: float | None) -> None:
    record = records[index]
    phase_count = 2 if duration >= 60 else 1
    phase_durations = [duration // 2, duration - duration // 2] if phase_count == 2 else [duration]
    position = scenario["start_position"][:]
    session_no = 0
    chaos_done = False
    for phase_no, phase_seconds in enumerate(phase_durations):
        phase_start = start_time + sum(phase_durations[:phase_no])
        phase_end = phase_start + phase_seconds
        while time.time() < phase_start and not stop.is_set():
            stop.wait(min(0.5, phase_start - time.time()))
        if stop.is_set():
            return
        if phase_no == 0:
            delay = random.Random((index + 1) * 104729 + int(start_time)).uniform(0, 8)
            timeline("bot_join_delay", {"bot": index + 1, "delay_s": round(delay, 1)})
            if stop.wait(delay):
                return
        retries = 0
        while time.time() < phase_end and not stop.is_set():
            session_no += 1
            remaining = max(5, int(phase_end - time.time() - 3))
            script = run_dir / "bots" / f"bot-{index + 1:02d}-session-{session_no:02d}.txt"
            position = make_bot_script(index, record, records, scenario, remaining, phase_no == 0 and session_no == 1, position, script)
            bot_name = f"Bot{index + 1:02d}"
            args = bot_args(bin_dir / ("osfr-bot.exe" if os.name == "nt" else "osfr-bot"), gateway_port, store, handoff, record, bot_name, script)
            stdout_path = run_dir / "bots" / f"bot-{index + 1:02d}-session-{session_no:02d}.stdout.log"
            stderr_path = run_dir / "bots" / f"bot-{index + 1:02d}-session-{session_no:02d}.stderr.log"
            stdout_path.parent.mkdir(parents=True, exist_ok=True)
            with stdout_path.open("w", encoding="utf-8", buffering=1) as out, stderr_path.open("w", encoding="utf-8", buffering=1) as err:
                proc = subprocess.Popen(args, cwd=run_dir, stdout=out, stderr=err,
                    stdin=subprocess.PIPE if "private_seed" in record else subprocess.DEVNULL)
                if proc.stdin is not None:
                    try:
                        proc.stdin.write(record["private_seed"])
                    except BrokenPipeError:
                        pass  # The ordinary bot result records early profile/config refusal.
                    finally:
                        proc.stdin.close()
                with registry_lock:
                    registry[record["guid"]] = proc
                timeline("bot_session_started", {"bot": index + 1, "session": session_no, "phase": phase_no + 1, "pid": proc.pid})
                forced = False
                while proc.poll() is None:
                    if stop.is_set():
                        proc.terminate()
                        forced = True
                        break
                    if index == 0 and chaos_time and not chaos_done and time.time() >= chaos_time:
                        proc.terminate()
                        forced = True
                        chaos_done = True
                        timeline("forced_disconnect", {"bot": 1, "pid": proc.pid, "playtest_elapsed_s": round(time.time() - start_time, 1)})
                        break
                    time.sleep(0.2)
                if forced and proc.poll() is None:
                    try:
                        proc.wait(timeout=3)
                    except subprocess.TimeoutExpired:
                        proc.kill()
                result = proc.wait()
                with registry_lock:
                    if registry.get(record["guid"]) is proc:
                        registry.pop(record["guid"], None)
            timeline("bot_session_ended", {"bot": index + 1, "session": session_no, "returncode": result, "forced": forced})
            if stop.is_set():
                return
            if forced and index == 0 and chaos_done:
                probe_path = run_dir / "bots" / "duplicate-session-probe.txt"
                probe_path.write_text("chat stress_reconnect_probe\nexpect-text stress_reconnect_probe\nquit\n", encoding="utf-8")
                probe_log = run_dir / "bots" / "duplicate-session-probe.log"
                probe_args = bot_args(bin_dir / ("osfr-bot.exe" if os.name == "nt" else "osfr-bot"), gateway_port, store, handoff, record, bot_name, probe_path)
                try:
                    # Login, ready, and the script expect each have their own deadline.
                    probe = run_bot(probe_args, record, run_dir, 3 * MAX_EXPECT_MS / 1000 + 10)
                except subprocess.TimeoutExpired as error:
                    partial = error.stdout or ""
                    partial = partial.decode("utf-8", errors="replace") if isinstance(partial, bytes) else partial
                    probe = subprocess.CompletedProcess(probe_args, 124, partial, "reconnect probe timed out\n")
                probe_log.write_text(probe.stdout + probe.stderr, encoding="utf-8")
                denied = probe.returncode != 0
                timeline("immediate_reconnect_probe", {"bot": 1, "returncode": probe.returncode, "refused": denied, "playtest_elapsed_s": round(time.time() - start_time, 1)})
                if denied:
                    with probe_log.open("a", encoding="utf-8") as log:
                        log.write("\nWaiting 65 seconds for the 60-second duplicate-session timeout.\n")
                    wait_until = time.time() + 65
                    while time.time() < wait_until and not stop.is_set():
                        stop.wait(min(1.0, wait_until - time.time()))
                if stop.is_set():
                    return
            if result != 0 and not forced:
                retries += 1
                timeline("bot_error", {"bot": index + 1, "session": session_no, "returncode": result, "retry": retries})
                bot_output = "\n".join(read_lines(stdout_path) + read_lines(stderr_path))
                duplicate_refused = "no SessionResponse from gateway" in bot_output or "duplicate session" in bot_output.lower()
                if duplicate_refused:
                    timeline("duplicate_session_refusal", {"bot": index + 1, "session": session_no, "returncode": result})
                    stop.wait(65)
                else:
                    stop.wait(min(5 * retries, max(0, phase_end - time.time())))
            else:
                retries = 0
            if result == 0 and not forced:
                state.setdefault("phase_saves", {}).setdefault(str(record["guid"]), []).append(read_save_state(run_dir, record["guid"], scenario["item_id"]))
                if time.time() >= phase_end - 10:
                    break
            if time.time() < phase_end and not stop.is_set():
                # The generated script fills the phase; this short cushion absorbs runtime jitter.
                stop.wait(min(0.25, phase_end - time.time()))


def read_save_state(run_dir: Path, guid: int, item_id: int) -> dict:
    path = run_dir / "state" / "characters" / f"{guid}.json"
    try:
        doc = load_json(path)
    except Exception as exc:
        return {"path": str(path), "error": str(exc)}
    found = []
    def walk(value):
        if isinstance(value, dict):
            if "coins" in value and isinstance(value.get("items"), list):
                found.append(value)
            for child in value.values():
                walk(child)
        elif isinstance(value, list):
            for child in value:
                walk(child)
    walk(doc)
    wallet = found[0] if found else {}
    count = 0
    for item in wallet.get("items", []):
        if isinstance(item, dict) and int(item.get("definition_id", item.get("item_id", -1))) == item_id:
            count += int(item.get("count", 0))
    return {"path": str(path), "coins": wallet.get("coins"), "item_count": count, "save_exists": path.is_file()}


class ProcessSampler:
    def __init__(self, csv_path: Path):
        self.csv = csv_path.open("w", newline="", encoding="utf-8")
        self.writer = csv.DictWriter(self.csv, fieldnames=["elapsed_s", "component", "pid", "rss_mb", "cpu_s", "cpu_pct", "error"])
        self.writer.writeheader()
        self.previous: dict[int, tuple[float, float]] = {}
        self.last = time.time()

    def sample_process(self, name: str, proc: subprocess.Popen) -> None:
        if proc.poll() is not None:
            return
        if os.name != "nt":
            self.writer.writerow({"elapsed_s": round(time.time() - RUN_START, 1), "component": name, "pid": proc.pid, "error": "Win32 process counters unavailable"})
            return
        try:
            import ctypes.wintypes as wt
            kernel = ctypes.WinDLL("kernel32", use_last_error=True)
            psapi = ctypes.WinDLL("psapi", use_last_error=True)
            class PMC(ctypes.Structure):
                _fields_ = [("cb", wt.DWORD), ("PageFaultCount", wt.DWORD), ("PeakWorkingSetSize", ctypes.c_size_t), ("WorkingSetSize", ctypes.c_size_t), ("QuotaPeakPagedPoolUsage", ctypes.c_size_t), ("QuotaPagedPoolUsage", ctypes.c_size_t), ("QuotaPeakNonPagedPoolUsage", ctypes.c_size_t), ("QuotaNonPagedPoolUsage", ctypes.c_size_t), ("PagefileUsage", ctypes.c_size_t), ("PeakPagefileUsage", ctypes.c_size_t), ("PrivateUsage", ctypes.c_size_t)]
            class FT(ctypes.Structure):
                _fields_ = [("dwLowDateTime", wt.DWORD), ("dwHighDateTime", wt.DWORD)]
            kernel.OpenProcess.restype = wt.HANDLE
            kernel.OpenProcess.argtypes = [wt.DWORD, wt.BOOL, wt.DWORD]
            kernel.GetProcessTimes.argtypes = [wt.HANDLE, ctypes.POINTER(FT), ctypes.POINTER(FT), ctypes.POINTER(FT), ctypes.POINTER(FT)]
            kernel.GetProcessTimes.restype = wt.BOOL
            kernel.CloseHandle.argtypes = [wt.HANDLE]
            psapi.GetProcessMemoryInfo.argtypes = [wt.HANDLE, ctypes.POINTER(PMC), wt.DWORD]
            psapi.GetProcessMemoryInfo.restype = wt.BOOL
            handle = kernel.OpenProcess(0x1000 | 0x0010, False, proc.pid)
            if not handle:
                raise OSError(ctypes.get_last_error(), "OpenProcess")
            counters = PMC()
            counters.cb = ctypes.sizeof(PMC)
            creation, exit_time, kernel_time, user_time = FT(), FT(), FT(), FT()
            if not psapi.GetProcessMemoryInfo(handle, ctypes.byref(counters), counters.cb):
                raise OSError(ctypes.get_last_error(), "GetProcessMemoryInfo")
            if not kernel.GetProcessTimes(handle, ctypes.byref(creation), ctypes.byref(exit_time), ctypes.byref(kernel_time), ctypes.byref(user_time)):
                raise OSError(ctypes.get_last_error(), "GetProcessTimes")
            kernel.CloseHandle(handle)
            cpu = (((kernel_time.dwHighDateTime << 32) | kernel_time.dwLowDateTime) + ((user_time.dwHighDateTime << 32) | user_time.dwLowDateTime)) / 10_000_000
            tick = time.time()
            old = self.previous.get(proc.pid)
            pct = ""
            if old and tick > old[0]:
                pct = round(max(0.0, cpu - old[1]) / (tick - old[0]) * 100.0, 2)
            self.previous[proc.pid] = (tick, cpu)
            self.writer.writerow({"elapsed_s": round(tick - RUN_START, 1), "component": name, "pid": proc.pid, "rss_mb": round(counters.WorkingSetSize / 1048576, 2), "cpu_s": round(cpu, 3), "cpu_pct": pct, "error": ""})
        except Exception as exc:
            self.writer.writerow({"elapsed_s": round(time.time() - RUN_START, 1), "component": name, "pid": proc.pid, "error": str(exc)})
        self.csv.flush()

    def close(self):
        self.csv.close()


RUN_START = 0.0


def sample_save_load(gateway_exe: Path, bin_dir: Path, records: list[dict], verify_port: int, store: Path, handoff: Path, reference: Path, content: Path, state_path: Path, run_dir: Path, trace: bool = False) -> tuple[ProcessLogs, list[dict]]:
    name = "gateway-verify"
    process = ProcessLogs(name, gateway_exe, gateway_args(verify_port, reference, content, state_path, store, handoff, run_dir, "gateway-verify.trace.jsonl" if trace else None), run_dir, run_dir)
    wait_ready(process, run_dir, ('"event":"listening"',))
    results = []
    for record in records:
        script = run_dir / "bots" / f"save-load-{record['guid']}.txt"
        script.write_text("wait 200\nquit\n", encoding="utf-8")
        args = bot_args(bin_dir / ("osfr-bot.exe" if os.name == "nt" else "osfr-bot"), verify_port, store, handoff, record, f"Load{record['guid']}", script)
        result = run_bot(args, record, run_dir, 20)
        (run_dir / "bots" / f"save-load-{record['guid']}.log").write_text(result.stdout + result.stderr, encoding="utf-8")
        results.append({"guid": record["guid"], "returncode": result.returncode, "loaded": result.returncode == 0, "error": "" if result.returncode == 0 else result.stderr[-1000:]})
    return process, results


def read_lines(path: Path) -> list[str]:
    try:
        return path.read_text(encoding="utf-8", errors="replace").splitlines()
    except OSError:
        return []


def scan_matches(paths: list[Path], pattern: re.Pattern, keep: int = 60) -> tuple[int, list[str]]:
    count = 0
    selected = []
    for path in paths:
        try:
            with path.open("r", encoding="utf-8", errors="replace") as source:
                for line in source:
                    if pattern.search(line):
                        count += 1
                        if len(selected) < keep:
                            selected.append(line.rstrip("\r\n")[:4000])
        except OSError:
            continue
    return count, selected


def capture_error_events(run_dir: Path) -> int:
    pattern = re.compile(r"panic|panicked|fatal|fail|error|refus|reject|denied|timeout|timed out|invalid packet|expired|duplicate session|sessionresponse|not online|mismatch|checksum", re.I)
    candidates = [
        run_dir / name for name in (
            "gateway.stdout.log", "gateway.stderr.log", "gateway-verify.stdout.log", "gateway-verify.stderr.log",
            "login.stdout.log", "login.stderr.log", "gateway.trace.jsonl", "gateway-verify.trace.jsonl",
            "login.trace.jsonl", "security.jsonl",
        )
    ]
    candidates.extend(sorted((run_dir / "bots").glob("*.stdout.log")))
    candidates.extend(sorted((run_dir / "bots").glob("*.stderr.log")))
    collected = []
    for path in candidates:
        try:
            with path.open("r", encoding="utf-8", errors="replace") as source:
                for number, line in enumerate(source, 1):
                    if not pattern.search(line):
                        continue
                    try:
                        event = json.loads(line)
                    except (json.JSONDecodeError, TypeError):
                        event = None
                    if isinstance(event, dict):
                        details = {key: event[key] for key in (
                            "event", "code", "guid", "account_id", "npc_id", "quest_id", "item_id",
                            "outcome", "reason", "error", "message", "returncode", "peer", "category", "kind",
                        ) if key in event}
                        summary = json.dumps(details, sort_keys=True, ensure_ascii=False)
                        collected.append(f"{path.relative_to(run_dir)}:{number}: {summary}")
                    else:
                        collected.append(f"{path.relative_to(run_dir)}:{number}: {line[:4000].rstrip()}")
        except OSError:
            continue
    (run_dir / "refusals-and-errors.log").write_text("\n".join(collected) + ("\n" if collected else ""), encoding="utf-8")
    return len(collected)


def latency_samples(run_dir: Path) -> list[float]:
    pattern = re.compile(r"^\[[^\]]+\s+(\d+)ms\]\s+(.*)$")
    samples = []
    for path in (run_dir / "bots").glob("bot-*.stdout.log"):
        pending: dict[str, int] = {}
        try:
            with path.open("r", encoding="utf-8", errors="replace") as source:
                for line in source:
                    match = pattern.match(line)
                    if not match:
                        continue
                    stamp = int(match.group(1))
                    body = match.group(2)
                    if "-> chat \"stressping_" in body:
                        token_match = re.search(r"stressping_\d+", body)
                        if token_match:
                            pending[token_match.group(0)] = stamp
                    elif "ok expect-text \"stressping_" in body:
                        token_match = re.search(r"stressping_\d+", body)
                        if token_match and token_match.group(0) in pending:
                            samples.append(max(0.0, (stamp - pending.pop(token_match.group(0))) / 1000.0))
        except OSError:
            continue
    return samples


def percentile(values: list[float], pct: float) -> float | None:
    if not values:
        return None
    ordered = sorted(values)
    index = max(0, min(len(ordered) - 1, math.ceil(pct * len(ordered)) - 1))
    return ordered[index]


def save_summary(run_dir: Path, records: list[dict], scenario: dict, phase_saves: dict) -> list[dict]:
    saves_dir = run_dir / "saves" / "end"
    saves_dir.mkdir(parents=True, exist_ok=True)
    results = []
    for record in records:
        source = run_dir / "state" / "characters" / f"{record['guid']}.json"
        if source.is_file():
            shutil.copy2(source, saves_dir / source.name)
        state = read_save_state(run_dir, record["guid"], scenario["item_id"])
        phases = phase_saves.get(str(record["guid"]), [])
        counts = [entry.get("item_count") for entry in phases if entry.get("item_count") is not None]
        purchase_observed = any(count >= 1 for count in counts) or state.get("item_count", 0) >= 1
        item_persisted = purchase_observed and len(counts) >= 2 and all(count == counts[0] for count in counts[1:]) and counts[0] >= 1
        coins_ok = isinstance(state.get("coins"), int) and state["coins"] >= 0
        results.append({"guid": record["guid"], "exists": source.is_file(), "coins": state.get("coins"), "coins_nonnegative": coins_ok, "purchased_item_count": state.get("item_count"), "purchase_observed": purchase_observed, "purchase_persisted_across_reconnect": item_persisted, "phase_samples": phases})
    for path in (run_dir / "state").rglob("*.json"):
        if path.is_file():
            relative = path.relative_to(run_dir / "state")
            target = saves_dir / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(path, target)
    return results


def report(run_dir: Path, metadata: dict, gateway_exit: int | None, login_exit: int | None, save_load: list[dict], saves: list[dict], latency: list[float], perf_rows: list[dict], timeline_events: list[dict]) -> str:
    panic_pattern = re.compile(r"panic|panicked|fatal error|stack overflow", re.I)
    panic_count, panic_lines = scan_matches([
        run_dir / "gateway.stdout.log", run_dir / "gateway.stderr.log",
        run_dir / "gateway-verify.stdout.log", run_dir / "gateway-verify.stderr.log", run_dir / "login.stderr.log",
    ], panic_pattern)
    error_events = read_lines(run_dir / "refusals-and-errors.log")
    checksum_mismatches = [line for line in error_events if "SOE checksum mismatch" in line]
    scenario = metadata.get("scenario", {})
    crash_events = [event for event in timeline_events if event["event"] in ("gateway_crash", "login_crash")]
    security_lines = read_lines(run_dir / "security.jsonl")
    cpu_values = [float(row["cpu_pct"]) for row in perf_rows if row.get("cpu_pct") not in (None, "", "NA")]
    bot_sessions = sum(event["event"] == "bot_session_started" for event in timeline_events)
    bot_errors = sum(event["event"] == "bot_error" for event in timeline_events)
    duplicate_refusals = sum(event["event"] == "duplicate_session_refusal" for event in timeline_events)
    immediate_refusals = sum(event["event"] == "immediate_reconnect_probe" and event.get("refused") for event in timeline_events)
    observed_purchases = [row for row in saves if row.get("purchase_observed")]
    rss_by_component: dict[str, list[float]] = {}
    for row in perf_rows:
        try:
            rss_by_component.setdefault(str(row["component"]), []).append(float(row["rss_mb"]))
        except (KeyError, TypeError, ValueError):
            continue
    lines = [
        "# W107 scratch gateway stress run",
        "",
        f"Run: `{run_dir.name}`  ",
        f"Build commit: `{metadata['commit']}`  ",
        f"Working-tree overlay: {metadata.get('working_tree', False)}; packet tracing: {metadata.get('trace', True)}.  ",
        f"Requested: {metadata['bots']} bots for {metadata['duration_seconds']} seconds; elapsed {metadata.get('elapsed_seconds', 0):.1f} seconds  ",
        f"Bot handshake/expect timeout: {metadata.get('expect_timeout_ms', MAX_EXPECT_MS)} ms; initial join stagger: 0 to {metadata.get('startup_stagger_max_seconds', 8)} seconds  ",
        f"Scenario: zone 0; quest {scenario.get('quest_id')}; combat targets from quest {scenario.get('combat_quest_id')}; vendor {scenario.get('vendor_id')}; item {scenario.get('item_id')} at {scenario.get('item_price')} coins; {len(scenario.get('enemies', []))} matching field-enemy rows",
        "",
        "## Summary",
        "",
        f"- Service exit codes: gateway `{gateway_exit}`, login `{login_exit}`.",
        f"- Bot sessions started: {bot_sessions}; bot errors: {bot_errors}.",
        f"- Broker login: {metadata.get('broker_login', False)}; broker refusals: {login_summary(run_dir)['broker_refusals']}; login failures: {login_summary(run_dir)['login_failures']}.",
        f"- Duplicate-session refusals: {duplicate_refusals}; immediate reconnect probe refusals: {immediate_refusals}.",
        f"- Captured refusal/error events across service, bot, trace, and security logs: {len(error_events)}; security-log rows: {len(security_lines)}.",
        f"- SOE checksum mismatch rejections indexed: {len(checksum_mismatches) if metadata.get('trace', True) else 'unavailable (packet tracing disabled; not a zero-rejection claim)'}.",
        f"- Service crash events: {len(crash_events)}; panic markers: {panic_count}.",
        f"- Unique ping round trips: {len(latency)}; p50 `{percentile(latency, .50)}` s, p95 `{percentile(latency, .95)}` s, p99 `{percentile(latency, .99)}` s.",
        f"- End saves present: {sum(row['exists'] for row in saves)}/{len(saves)}; gateway-loader reconnects: {sum(row['loaded'] for row in save_load)}/{len(save_load)}.",
        f"- Nonnegative coin checks: {sum(row['coins_nonnegative'] for row in saves)}/{len(saves)}; scripted item purchases observed: {len(observed_purchases)}/{len(saves)}; purchase persistence checks passed: {sum(row['purchase_persisted_across_reconnect'] for row in observed_purchases)}/{len(observed_purchases)}.",
        "",
        "## Timeline",
        "",
    ]
    for event in timeline_events:
        details = [f"{key}={event[key]}" for key in ("bot", "session", "phase", "pid", "delay_s", "returncode", "refused", "playtest_elapsed_s", "gateway_port", "login_port", "auth_port") if key in event]
        suffix = "; " + ", ".join(details) if details else ""
        lines.append(f"- +{event.get('elapsed_s', 0):.1f}s `{event['event']}`{suffix}")
    lines.extend(["", "## Crashes, stalls, refusals, and slow steps", ""])
    if panic_count:
        lines.extend(["Panic/crash log lines:", "", "```text", *panic_lines[:60], "```", ""])
    else:
        lines.extend(["No panic or crash marker appeared in captured service output.", ""])
    if crash_events:
        lines.extend(["Service crash events:", "", *[f"- `{event['event']}` at +{event.get('elapsed_s', 0):.1f}s, return code `{event.get('returncode')}`." for event in crash_events], ""])
    if error_events:
        lines.extend(["Refusal/error event excerpts (complete index: `refusals-and-errors.log`):", "", "```text", *error_events[:40], "```", ""])
        if len(error_events) > 40:
            lines.append(f"{len(error_events) - 40} additional events are in the index.")
    lines.append("Complete stdout/stderr and JSON traces remain in this run folder.")
    lines.append("")
    if security_lines:
        lines.append("Security log entries are preserved in `security.jsonl`; they record rejected packet/admission summaries only.")
    else:
        lines.append("`security.jsonl` was empty; no security-log refusal event was recorded.")
    lines.extend(["", "## Process resource samples", ""])
    if rss_by_component:
        lines.append("| Process | Samples | First RSS MiB | Last RSS MiB | Min RSS MiB | Max RSS MiB |")
        lines.append("|---|---:|---:|---:|---:|---:|")
        for name, values in sorted(rss_by_component.items()):
            lines.append(f"| {name} | {len(values)} | {values[0]:.2f} | {values[-1]:.2f} | {min(values):.2f} | {max(values):.2f} |")
        lines.append("")
        lines.append(f"Maximum sampled process CPU: `{max(cpu_values):.2f}%` per process." if cpu_values else "CPU counters were unavailable in the samples.")
    else:
        lines.append("No process resource samples were recorded.")
    lines.extend(["", "## Save integrity", ""])
    for row in saves:
        phase_counts = [entry.get("item_count") for entry in row.get("phase_samples", [])]
        lines.append(f"- GUID {row['guid']}: save={'present' if row['exists'] else 'missing'}, coins={row['coins']}, coins_nonnegative={row['coins_nonnegative']}, item_count={row['purchased_item_count']}, phase item counts={phase_counts}, purchase_observed={row.get('purchase_observed', False)}, reconnect_purchase_persisted={row['purchase_persisted_across_reconnect']}.")
    lines.append("")
    for row in save_load:
        if not row["loaded"]:
            lines.append(f"- Gateway loader failed for GUID {row['guid']}: `{row['error']}`.")
    if not any(not row["loaded"] for row in save_load):
        lines.append("All saved characters reconnected after a gateway restart; the gateway's own gameplay/save loader accepted them.")
    lines.extend(["", "## Ranked findings", ""])
    findings = []
    if panic_count:
        findings.append(("P1", "Service panic/crash", "See the exact panic lines above. Reproduce by rerunning the same bot scenario and action sequence; logs and bot scripts are retained."))
    elif crash_events:
        findings.append(("P1", "Service crash", "The service exited during the run; exact exit code and timeline are recorded above. Logs and bot scripts are retained."))
    if any(not row["loaded"] for row in save_load):
        findings.append(("P1", "Saved state rejected on reload", "Reconnect the listed GUID using `bots/save-load-<guid>.txt` against the preserved run state."))
    if any(not row["coins_nonnegative"] for row in saves):
        findings.append(("P1", "Coin save invariant failed", "Inspect `state/characters/<guid>.json` and the shop transaction in gateway trace."))
    if observed_purchases and any(not row["purchase_persisted_across_reconnect"] for row in observed_purchases):
        findings.append(("P1", "Purchased item did not persist across reconnect", "Compare the purchase packet in the bot session log with the phase snapshots and end save for the same GUID."))
    elif not observed_purchases:
        findings.append(("P2", "Purchase persistence gate not exercised", "No scripted item purchase appears in the phase or end saves; resolve the bot transport/action failures before drawing a save-persistence conclusion."))
    if checksum_mismatches:
        findings.append(("P1", "Gateway rejected bot transport packets with SOE checksum mismatches", f"{len(checksum_mismatches)} rejected packets are indexed in `refusals-and-errors.log`; inspect the matching trace rows and bot session logs."))
    if bot_errors:
        rank = "P1" if bot_errors * 2 >= max(1, bot_sessions) else "P2"
        findings.append((rank, "Bot sessions repeatedly failed before completing play", f"{bot_errors}/{bot_sessions} sessions returned errors; compare per-session bot stdout with gateway trace entries for reliable-send retries, timeouts, and rejected packets."))
    if any(event["event"] == "duplicate_session_refusal" or (event["event"] == "immediate_reconnect_probe" and event.get("refused")) for event in timeline_events):
        findings.append(("P2", "Duplicate-session refusal observed", "The forced bot disconnect was followed by an immediate reconnect probe; the harness waited 65 seconds before retrying."))
    if not findings:
        findings.append(("Info", "No repeatable crash or persistence defect found in this run", "Review refusal/error rows and gateway trace before treating the gate as clean."))
    rank_order = {"P0": 0, "P1": 1, "P2": 2, "P3": 3}
    for rank, title, reproduction in sorted(findings, key=lambda item: rank_order.get(item[0], 4)):
        lines.append(f"- **{rank} — {title}.** {reproduction}")
    lines.extend([
        "",
        "## Evidence boundaries and open gaps",
        "",
        ("- Every bot session, including save reload, uses AUTH2, native Session login and character-select handoff. Profiles contain key paths and trusted identity; no HMAC key is given to bots."
            if metadata.get("broker_login") else "- Bot actions use the direct gateway HMAC-ticket path; login/broker services are idle."),
        "- Accounts were created by the `osfr-account` CLI. Synthetic character identity/world-state records were added to the scratch account store because no standalone character-creation CLI exists; the gateway loader validated them at startup/reconnect. Native login UI character creation was not exercised.",
        "- The original packet layouts and client behavior are distinguished from local policy in the source and native notes. Quest/enemy gating, starting coordinates and wallet seed, movement pacing, stock mapping, and reconstructed quest rewards remain server/content policy (R), not recovered retail rules.",
        "- Field-enemy placements are reconstructed zone-0 data in `notes/Design/Field Enemies.md`; this run does not establish historical spawn geometry or production balance.",
        "- Native-client rendering, collision, attack damage/effects, real friend/group acceptance, launcher credential flow, external network behavior, and persistence under power loss were not verified here.",
        "- No source code, repository content, launcher, or native client was changed/launched.",
        "",
        "## Provenance",
        "",
        f"- Source build: `git archive HEAD` at commit `{metadata['commit']}`; command output is retained in `build.stdout.log` and `build.stderr.log`.",
        "- Gateway, login, bot, and account CLI implementations: `prototype/osfr-tools/src/bin/osfr-gateway.rs`, `osfr-login.rs`, `osfr-bot.rs`, `osfr-account.rs`.",
        "- Bot actions and multiplayer helper patterns: `prototype/osfr-tools/tests/multiplayer.rs`, `tests/bot_smoke.rs`.",
        "- NPC 26/8 request provenance comment points to `lab/runs/client-baseline-20261009T103605Z/analysis/application-0066.bin`; reconstructed interaction option meanings are documented in `src/npc_interaction.rs`.",
        "- Quest packet provenance is in `src/npc_interaction.rs` and the native quest reply capture references in `tests/npc_interaction.rs`; quest acceptance policy remains reconstructed.",
        "- Zone-0 Hooligan quest/enemy association: `notes/Design/Field Enemies.md` and scratch `content/field_enemies.json`; content tags distinguish client/legacy/reconstructed fields.",
        "- Runtime content started from the same `git archive HEAD` used for the build, then received the preserved `lab/reports/round70-native-20261016T061000Z/content` overlay. Only the scratch copy's starting-wallet coins were adjusted to 5,000 (R).",
    ])
    return "\n".join(lines) + "\n"


def main() -> int:
    global RUN_START
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bots", type=int, default=10)
    parser.add_argument("--duration-seconds", type=int, default=3600)
    parser.add_argument("--trace", action="store_true", help="opt in to synchronous per-packet tracing (affects latency)")
    parser.add_argument("--broker-login", action="store_true", help="AUTH2, native login and character-select handoff for every bot")
    parser.add_argument("--working-tree", action="store_true", help="freeze current crate sources over the HEAD snapshot")
    args = parser.parse_args()
    if not 1 <= args.bots <= 10:
        parser.error("--bots must be between 1 and 10")
    if args.duration_seconds < 1:
        parser.error("--duration-seconds must be positive")

    # Broker mode must exercise the owned uncommitted bot implementation.
    args.working_tree = args.working_tree or args.broker_login
    script_path = Path(__file__).resolve()
    root = script_path.parents[2]
    lab = root.parent / "lab"
    build_dir = Path(os.environ.get("CARGO_TARGET_DIR", str(lab / "build" / "w107"))).resolve()
    # stage_head may recursively remove only this exact child of the lab build root.
    if build_dir.parent != (lab / "build").resolve():
        raise RuntimeError("CARGO_TARGET_DIR must name one directory directly under lab/build")
    reports = lab / "reports" / ("broker-login-w130" if args.broker_login else "stress-w124")
    run_dir = reports / utc_stamp()
    run_dir.mkdir(parents=True, exist_ok=False)
    build_dir.mkdir(parents=True, exist_ok=True)
    RUN_START = time.time()
    timeline_events: list[dict] = []
    timeline_lock = threading.Lock()

    def timeline(event: str, fields: dict | None = None):
        record = {"timestamp_utc": now_utc().isoformat(), "elapsed_s": round(time.time() - RUN_START, 1) if RUN_START else 0, "event": event, **(fields or {})}
        with timeline_lock:
            timeline_events.append(record)
            with (run_dir / "timeline.jsonl").open("a", encoding="utf-8") as output:
                output.write(json.dumps(record, sort_keys=True) + "\n")

    commit_result = run_command(["git", "rev-parse", "HEAD"], root)
    if commit_result.returncode:
        raise RuntimeError("git rev-parse HEAD failed")
    commit = commit_result.stdout.strip()
    (run_dir / "build.commit.txt").write_text(commit + "\n", encoding="ascii")
    metadata = {"commit": commit, "bots": args.bots, "duration_seconds": args.duration_seconds, "expect_timeout_ms": MAX_EXPECT_MS, "startup_stagger_max_seconds": 8, "trace": args.trace, "working_tree": args.working_tree, "broker_login": args.broker_login}
    login_process = gateway_process = None
    records = []
    bot_registry: dict[int, subprocess.Popen] = {}
    registry_lock = threading.Lock()
    stop = threading.Event()
    perf = ProcessSampler(run_dir / "processes.csv")
    save_load: list[dict] = []
    save_rows: list[dict] = []
    samples: list[dict] = []
    service_exit: dict[str, int | None] = {"gateway": None, "login": None}
    playtest_start: float | None = None
    service_crashed = False

    try:
        timeline("build_started", {"commit": commit})
        gateway_exe = build_bins(root, build_dir, run_dir, commit, args.working_tree)
        bin_dir = build_dir / "release"
        account_exe = bin_dir / ("osfr-account.exe" if os.name == "nt" else "osfr-account")
        login_exe = bin_dir / ("osfr-login.exe" if os.name == "nt" else "osfr-login")
        head_content = build_dir / "head-src" / "prototype" / "osfr-tools" / "data" / "content"
        content = copy_runtime_content(head_content, root, run_dir)
        scenario = scenario_from_content(content)
        metadata["scenario"] = scenario
        store, records = create_accounts(account_exe, run_dir, scenario, args.bots, args.broker_login)
        metadata["account_count"] = len(records)
        write_login_resources(run_dir)
        reference = lab / "references" / "rust-gateway-reference"
        if not (reference / "manifest.json").is_file():
            raise RuntimeError(f"validated local gateway reference is missing: {reference}")
        handoff = run_dir / "handoff.key"
        login_key = run_dir / "login.key"
        server_id = run_dir / "server.id"
        handoff.write_bytes(os.urandom(32))
        login_key.write_bytes(os.urandom(16))
        server_id.write_bytes(os.urandom(32))
        state_path = run_dir / "state" / "gateway.json"
        login_port, auth_port, gateway_port = free_port("udp"), free_port("tcp"), free_port("udp")
        if args.broker_login:
            for index, record in enumerate(records):
                profile = run_dir / f"bot-{index + 1:02d}.profile.json"
                write_json(profile, {"handle": record["handle"], "key_file": "-",
                    "server_id_file": str(server_id), "broker": f"127.0.0.1:{auth_port}",
                    "login": f"127.0.0.1:{login_port}", "login_key_file": str(login_key)})
                record["broker_profile"] = profile
        login_process = ProcessLogs("login", login_exe, [
            "--bind", f"127.0.0.1:{login_port}", "--auth-bind", f"127.0.0.1:{auth_port}",
            "--server-id-file", str(server_id), "--resources", str(run_dir / "login-resources"),
            "--identity", str(run_dir / "identity.json"), "--state", str(run_dir / "login-state.json"),
            "--account-store", str(store), "--key-file", str(login_key), "--handoff-key-file", str(handoff),
            "--gateway", f"127.0.0.1:{gateway_port}",
        ] + (["--trace", str(run_dir / "login.trace.jsonl")] if args.trace else []), root, run_dir)
        wait_ready(login_process, run_dir, ('"event":"listening"', '"event":"auth_listening"'))
        gateway_process = ProcessLogs("gateway", gateway_exe, gateway_args(gateway_port, reference, content, state_path, store, handoff, run_dir, "gateway.trace.jsonl" if args.trace else None), root, run_dir)
        wait_ready(gateway_process, run_dir, ('"event":"listening"',))
        timeline("services_ready", {"login_port": login_port, "auth_port": auth_port, "gateway_port": gateway_port})

        playtest_start = time.time()
        duration = args.duration_seconds
        phase_boundary = playtest_start + duration / 2 if duration >= 600 else None
        chaos_time = playtest_start + duration * 0.25 if duration >= 600 else None
        workers = []
        for index in range(args.bots):
            thread = threading.Thread(
                target=bot_worker,
                args=(index, metadata, stop, bot_registry, registry_lock, timeline, duration, gateway_port, store, handoff, records, scenario, bin_dir, run_dir, playtest_start, chaos_time),
                daemon=True,
            )
            thread.start()
            workers.append(thread)
        timeline("playtest_started", {"bots": args.bots, "duration_seconds": duration})
        sampler_next = time.time()
        while time.time() < playtest_start + duration:
            if gateway_process.process.poll() is not None:
                timeline("gateway_crash", {"returncode": gateway_process.process.returncode})
                service_crashed = True
                break
            if login_process.process.poll() is not None:
                timeline("login_crash", {"returncode": login_process.process.returncode})
                service_crashed = True
                break
            if time.time() >= sampler_next:
                perf.sample_process("gateway", gateway_process.process)
                perf.sample_process("login", login_process.process)
                with registry_lock:
                    active = list(bot_registry.items())
                for guid, proc in active:
                    perf.sample_process(f"bot-{guid}", proc)
                sampler_next = time.time() + 30
            if phase_boundary and not metadata.get("phase_boundary_recorded") and time.time() >= phase_boundary:
                timeline("reconnect_phase_boundary")
                metadata["phase_boundary_recorded"] = True
            time.sleep(0.5)
        stop.set()
        for thread in workers:
            thread.join(timeout=10)
        with registry_lock:
            for proc in bot_registry.values():
                if proc.poll() is None:
                    proc.terminate()
        for thread in workers:
            thread.join(timeout=5)
        timeline("playtest_ended", {"playtest_elapsed_s": round(time.time() - playtest_start, 1) if playtest_start else 0})

        if service_crashed:
            timeline("save_reload_skipped_after_service_crash")

        # Every character is reloaded by a fresh gateway process through its real loader.
        if not service_crashed and gateway_process.process.poll() is None:
            gateway_process.process.terminate()
            try:
                gateway_process.process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                gateway_process.process.kill()
                gateway_process.process.wait(timeout=5)
        gateway_process.close_logs()
        if not service_crashed:
            gateway_process, save_load = sample_save_load(gateway_exe, bin_dir, records, gateway_port if args.broker_login else free_port("udp"), store, handoff, reference, content, state_path, run_dir, args.trace)
            if gateway_process.process.poll() is None:
                gateway_process.process.terminate()
                try:
                    gateway_process.process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    gateway_process.process.kill()
                    gateway_process.process.wait(timeout=5)
            service_exit["gateway"] = 0
            gateway_process.close_logs()
        else:
            service_exit["gateway"] = gateway_process.process.returncode
        if login_process.process.poll() is None:
            login_process.process.terminate()
            try:
                login_process.process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                login_process.process.kill()
                login_process.process.wait(timeout=5)
        service_exit["login"] = 0 if not service_crashed else login_process.process.returncode
        login_process.close_logs()
        samples = []
        with (run_dir / "processes.csv").open("r", newline="", encoding="utf-8") as f:
            samples = list(csv.DictReader(f))
        save_rows = save_summary(run_dir, records, scenario, metadata.get("phase_saves", {}))
        capture_error_events(run_dir)
        timeline("save_reload_complete", {"loaded": sum(row["loaded"] for row in save_load), "total": len(save_load)})
        metadata["elapsed_seconds"] = round(time.time() - playtest_start, 1) if playtest_start else 0
        (run_dir / "REPORT.md").write_text(report(run_dir, metadata, service_exit["gateway"], service_exit["login"], save_load, save_rows, latency_samples(run_dir), samples, timeline_events), encoding="utf-8")
        authentication = login_summary(run_dir)
        print(json.dumps({
            "run_dir": str(run_dir), "commit": commit, "bots": args.bots, "duration_seconds": args.duration_seconds,
            "elapsed_seconds": metadata["elapsed_seconds"], "gateway_exit": service_exit["gateway"], "login_exit": service_exit["login"],
            "bot_errors": sum(event["event"] == "bot_error" for event in timeline_events),
            **authentication,
            "latency_p50_s": percentile(latency_samples(run_dir), .50),
            "latency_p95_s": percentile(latency_samples(run_dir), .95),
            "latency_p99_s": percentile(latency_samples(run_dir), .99),
            "latency_samples": len(latency_samples(run_dir)), "save_load": f"{sum(row['loaded'] for row in save_load)}/{len(save_load)}",
            "coins_nonnegative": f"{sum(row['coins_nonnegative'] for row in save_rows)}/{len(save_rows)}",
            "purchase_persisted": f"{sum(row['purchase_persisted_across_reconnect'] for row in save_rows)}/{len(save_rows)}",
        }, indent=2))
        return 1 if service_crashed or (args.broker_login and (
            any(event["event"] == "bot_error" for event in timeline_events)
            or authentication["broker_refusals"] or authentication["login_failures"]
            or any(not row["loaded"] for row in save_load)
        )) else 0
    except Exception as exc:
        timeline("runner_error", {"error": f"{type(exc).__name__}: {exc}"})
        capture_error_events(run_dir)
        metadata["elapsed_seconds"] = round(time.time() - playtest_start, 1) if playtest_start else 0
        (run_dir / "REPORT.md").write_text(report(run_dir, metadata, gateway_process.process.returncode if gateway_process else None, login_process.process.returncode if login_process else None, save_load, save_rows, latency_samples(run_dir), samples, timeline_events) + f"\nRunner error: `{type(exc).__name__}: {exc}`.\n", encoding="utf-8")
        raise
    finally:
        stop.set()
        with registry_lock:
            children = list(bot_registry.values())
        for proc in children:
            if proc.poll() is None:
                proc.terminate()
                try:
                    proc.wait(timeout=3)
                except subprocess.TimeoutExpired:
                    proc.kill()
        for process in (gateway_process, login_process):
            if process is not None:
                if process.process.poll() is None:
                    process.process.terminate()
                    try:
                        process.process.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        process.process.kill()
                        process.process.wait(timeout=3)
                process.close_logs()
        # Check every in-memory stress seed before dropping it; none are persisted.
        scan = scan_private_keys(run_dir, records) if args.broker_login else {"matches": []}
        if args.broker_login:
            write_json(run_dir / "secret-scan.json", scan)
            for record in records:
                record.pop("private_seed", None)
        for key_name in ("handoff.key", "login.key", "server.id"):
            (run_dir / key_name).unlink(missing_ok=True)
        perf.close()
        if scan["matches"]:
            raise RuntimeError("secret scan failed; see secret-scan.json")


if __name__ == "__main__":
    raise SystemExit(main())
