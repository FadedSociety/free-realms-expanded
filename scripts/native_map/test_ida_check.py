"""Headless IDA smoke test for ida_check.py; run through the ida-pro-mcp uv project."""

from __future__ import annotations

import hashlib
import json
import os
import sys
import unittest
from datetime import datetime, timezone
from pathlib import Path


SCRIPT_DIR = Path(__file__).resolve().parent
sys.dont_write_bytecode = True
sys.path.insert(0, str(SCRIPT_DIR))
import ida_check


SAMPLE = ida_check.LAB_ROOT / "build" / "w97" / "where.exe"


class Test00SamplingAndPathRules(unittest.TestCase):
    def test_sampling_is_confirmed_first_and_likely_is_seeded(self) -> None:
        records = [
            {"addr": f"{n:08x}", "record": {"confidence": confidence}}
            for n, confidence in [(1, "likely"), (2, "confirmed"), (3, "likely"),
                                  (4, "confirmed"), (5, "likely"), (6, "likely")]
        ]
        first = ida_check.select_labels(records, confirmed_count=1, likely_count=2, seed=97)
        again = ida_check.select_labels(records, confirmed_count=1, likely_count=2, seed=97)
        self.assertEqual([r["record"]["confidence"] for r in first], ["confirmed", "likely", "likely"])
        self.assertEqual([r["addr"] for r in first], [r["addr"] for r in again])

    def test_binary_path_is_limited_to_the_sample_or_client(self) -> None:
        target, is_client, _ = ida_check.validate_binary(SAMPLE)
        self.assertEqual(target, SAMPLE.resolve())
        self.assertFalse(is_client)
        outside = Path(os.environ["SystemRoot"]) / "System32" / "where.exe"
        with self.assertRaises(ValueError):
            ida_check.validate_binary(outside)
        with self.assertRaises(ValueError):
            ida_check.write_report("../outside", [], {})


class Test01IdaCheckSmoke(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.api, cls.is_client = ida_check.open_database(SAMPLE)

    @classmethod
    def tearDownClass(cls) -> None:
        ida_check.close_database(cls.api, save=not cls.is_client)

    def test_ida_boundary_decompilation_and_report_on_where_copy(self) -> None:
        self.assertTrue(SAMPLE.is_file())
        self.assertTrue(self.api.hexrays_available, "Hex-Rays is required for this check")
        row = None
        for address in self.api.idautils.Functions():
            label = {
                "addr": f"{address:08x}",
                "status": "accepted",
                "record": {
                    "name": "W97SmokeFunction",
                    "confidence": "confirmed",
                    "kind": "function",
                    "subsystem": "unknown.smoke",
                    "does": "Synthetic label used only to exercise the IDA output path.",
                    "purpose": None,
                    "evidence": [],
                    "wire": [],
                },
            }
            map_function = {"bytes": 0, "callers": "[]", "callees": "[]", "strings": "[]"}
            candidate = ida_check.inspect_label(self.api, label, map_function)
            if candidate["ida"]["boundary_match"] and candidate["ida"]["pseudocode"]:
                row = candidate
                break

        self.assertIsNotNone(row, "where.exe must provide an exact function boundary and Hex-Rays pseudocode")
        self.assertEqual(row["ida"]["address"], row["source"]["address"])
        self.assertTrue(row["agreement"]["boundary_match"])

        sample_hash = hashlib.sha256(SAMPLE.read_bytes()).hexdigest()
        run_name = "w97-selftest-" + datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
        summary = ida_check.summarize(
            [row], target=SAMPLE, target_sha256=sample_hash, map_database=None,
            selection={"mode": "synthetic smoke test", "selected": 1},
            ida_version=self.api.version,
        )
        jsonl_path, summary_path = ida_check.write_report(run_name, [row], summary)
        emitted = json.loads(jsonl_path.read_text(encoding="utf-8").splitlines()[0])
        emitted_summary = json.loads(summary_path.read_text(encoding="utf-8"))
        self.assertEqual(emitted["source"]["address"], row["source"]["address"])
        self.assertEqual(emitted_summary["results"]["processed"], 1)
        self.assertGreater(emitted["ida"]["pseudocode_chars"], 0)


if __name__ == "__main__":
    unittest.main(verbosity=2)
