"""Negative tests for the playtest bundle's frozen activity gate."""

from __future__ import annotations

import json
import os
import shutil
import struct
import sys
import unittest
import uuid
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts" / "bundle"))

import bundle_common  # noqa: E402


class ActivityContentGateTest(unittest.TestCase):
    def setUp(self):
        base = Path(
            os.environ.get("OSFR_TEST_TMP", ROOT / "prototype" / "osfr-tools" / "target")
        )
        base.mkdir(parents=True, exist_ok=True)
        self.temp = base / f"osfr-activity-gate-{uuid.uuid4().hex}"
        self.bundle = self.temp / "bundle"
        self.content = self.bundle / "content"
        self.content.mkdir(parents=True)
        source = ROOT / "prototype" / "osfr-tools" / "data" / "content"
        for name in bundle_common.ACTIVITY_CONTENT:
            shutil.copy2(source / name, self.content / name)

    def tearDown(self):
        shutil.rmtree(self.temp)

    def read_json(self, name: str):
        return json.loads((self.content / name).read_text(encoding="utf-8-sig"))

    def refuse(self, name: str, mutate, message: str):
        value = self.read_json(name)
        mutate(value)
        path = self.content / name
        original = path.read_bytes()
        path.write_text(json.dumps(value), encoding="utf-8")
        before = path.read_bytes()
        try:
            with self.assertRaisesRegex(ValueError, message):
                bundle_common.check_activity_content(self.bundle)
            self.assertEqual(path.read_bytes(), before, "a refusal changed the bundle")
        finally:
            path.write_bytes(original)

    def test_reviewed_bindings_are_refused(self):
        bundle_common.check_activity_content(self.bundle)

        def replace_word(value, offset, word):
            value["details_loading"][offset:offset + 4] = word.to_bytes(4, "little")

        def replace_bytes(value, offset, payload):
            value["details_loading"][offset:offset + len(payload)] = payload

        self.refuse(
            "kart_entry.json",
            lambda value: replace_word(value, 4, value["context"]["encounter_id"] + 1),
            "41/114 frame",
        )
        self.refuse(
            "kart_entry.json",
            lambda value: replace_word(value, 8, value["context"]["instance_id"] + 1),
            "41/114 frame",
        )
        self.refuse(
            "kart_entry.json",
            lambda value: replace_word(value, 16, 3),
            "41/114 frame",
        )
        self.refuse(
            "kart_entry.json",
            lambda value: replace_bytes(
                value, 32, (value["context"]["session"] + 1).to_bytes(8, "little")
            ),
            "41/114 frame",
        )
        self.refuse(
            "kart_entry.json",
            lambda value: replace_bytes(value, 51, struct.pack("<f", 1.0)),
            "41/114 frame",
        )
        self.refuse(
            "kart_lobby.json",
            lambda value: value["games"][0].pop("name_id"),
            "invalid row",
        )
        self.refuse(
            "mining.json",
            lambda value: value["spec"]["goals"][0].update(per_reward=0),
            "invalid goals",
        )

    def test_reviewed_scene_and_board_mutations_are_refused(self):
        bundle_common.check_activity_content(self.bundle)

        def replace_scene_float(value, index, replacement):
            value["context"]["scene"]["rotation"][index] = replacement

        def replace_shape_cell(value, index, replacement):
            value["spec"]["shape"][index] = replacement

        def flatten_shape(value):
            value["spec"]["shape"] = [0] * len(value["spec"]["shape"])

        self.refuse(
            "kart_entry.json",
            lambda value: replace_scene_float(value, 0, 1e100),
            "finite f32",
        )
        self.refuse(
            "mining.json",
            lambda value: replace_shape_cell(value, 0, 7),
            "invalid board shape code",
        )
        self.refuse("mining.json", flatten_shape, "no playable tiles")
        self.refuse(
            "mining.json",
            lambda value: replace_shape_cell(value, 11 * 14 + 3, 1),
            "complete dropzone row",
        )
        self.refuse(
            "mining.json",
            lambda value: value["spec"].update(tiles=[{}]),
            "invalid tile rows",
        )


if __name__ == "__main__":
    unittest.main()
