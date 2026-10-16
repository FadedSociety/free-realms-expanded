"""Headless IDA smoke test for ida_check.py; run through the ida-pro-mcp uv project."""

from __future__ import annotations

import hashlib
import json
import os
import sys
import unittest
from datetime import datetime, timezone
from pathlib import Path
from types import SimpleNamespace


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


class Test00PacketWriterHeuristic(unittest.TestCase):
    def test_w97_description_keyword_false_positives_are_not_packet_writers(self) -> None:
        false_positives = [
            {
                "name": "std::length_error::length_error",
                "does": "Constructor of std::length_error; overwrites the base vtable pointer.",
                "purpose": "Exception thrown when a requested length exceeds the container maximum.",
                "subsystem": "crt.stl",
            },
            {
                "name": "StampAlloc_0054d407",
                "does": "Allocator wrapper; writes the magic pair into a returned block.",
                "purpose": "Per-class pooled allocation entry; requested by its callers.",
                "subsystem": "engine.mem",
            },
        ]
        for record in false_positives:
            with self.subTest(name=record["name"]):
                self.assertFalse(ida_check._label_is_packet_writer(record))

    def test_packet_or_serializer_name_and_serializer_callee_are_recognized(self) -> None:
        self.assertTrue(ida_check._label_is_packet_writer({"name": "ClientPacketWriter"}))
        self.assertTrue(ida_check._label_is_packet_writer({"name": "RequestHandler"}, ["EncodeMessage"]))


class Test00ReviewEvidence(unittest.TestCase):
    @staticmethod
    def row() -> dict:
        edges = {"map": [], "ida": [], "shared": [], "map_only": [], "ida_only": [],
                 "map_edge_coverage": None, "jaccard": None}
        return {
            "source": {"address": "00400000", "confidence": "confirmed", "name": "Synthetic",
                       "subsystem": "net.packet.writer"},
            "ida": {"function_exists": True, "function_start": "00400000", "boundary_match": True,
                    "function_created_on_demand": False, "pseudocode": "int f()", "pseudocode_truncated": False},
            "agreement": {"boundary_match": True, "callers": dict(edges), "callees": dict(edges),
                          "strings": {"required": [], "missing": []}},
            "flags": [],
        }

    def test_review_queue_requires_both_call_graphs_to_be_disjoint(self) -> None:
        row = self.row()
        row["agreement"]["callers"].update(map=["00400010"], ida=["00400020"])
        self.assertIsNone(ida_check.review_queue_entry(row))
        row["agreement"]["callees"].update(map=["00400030"], ida=["00400040"])
        entry = ida_check.review_queue_entry(row)
        self.assertEqual(entry["severity"], "high")
        self.assertEqual(entry["reasons"][0]["code"], "caller_and_callee_graphs_disjoint")

    def test_summary_reports_boundary_rates_by_confidence_and_prefix(self) -> None:
        confirmed = self.row()
        likely = self.row()
        likely["source"].update(address="00400001", confidence="likely", subsystem="ui")
        summary = ida_check.summarize(
            [confirmed, likely], SAMPLE, "hash", None, {"selected": 2}, [9, 4, 0]
        )
        confirmed_stats = summary["agreement_rates_by_confidence"]["confirmed"]
        self.assertEqual(confirmed_stats["results"]["processed"], 1)
        self.assertEqual(confirmed_stats["agreement_rates"]["boundaries"]["rate"], 1.0)
        self.assertEqual(summary["agreement_rates_by_subsystem_prefix"]["net"]["results"]["processed"], 1)


class Test00OnDemandFunctionWithFake(unittest.TestCase):
    def test_code_without_function_is_added_in_memory(self) -> None:
        funcs = SimpleNamespace(function=None)
        def get_func(address):
            function = funcs.function
            return function if function and function.start_ea <= address < function.end_ea else None
        def add_func(address):
            funcs.function = SimpleNamespace(start_ea=address, end_ea=address + 8)
            return True
        funcs.get_func = get_func
        funcs.add_func = add_func
        api = SimpleNamespace(
            ida_funcs=funcs,
            ida_bytes=SimpleNamespace(get_flags=lambda _address: 1, is_code=lambda flags: flags == 1),
            ida_name=SimpleNamespace(get_name=lambda _address: ""),
            ida_nalt=SimpleNamespace(),
            ida_ua=SimpleNamespace(insn_t=object, decode_insn=lambda _insn, _address: 0),
            ida_idp=SimpleNamespace(is_call_insn=lambda _insn: False),
            idautils=SimpleNamespace(FuncItems=lambda _start: [], DataRefsFrom=lambda _address: [],
                                     CodeRefsFrom=lambda _address, _flow: [],
                                     CodeRefsTo=lambda _address, _flow: []),
            ida_hexrays=SimpleNamespace(),
            hexrays_available=False,
            imagebase=0x400000,
        )
        label = {"addr": "00400000", "status": "accepted", "record": {"name": "Synthetic"}}
        row = ida_check.inspect_label(api, label, {"bytes": 8, "callers": "[]", "callees": "[]", "strings": "[]"})
        self.assertTrue(row["ida"]["function_created_on_demand"])
        self.assertTrue(row["agreement"]["boundary_match"])


class Test01IdaCheckSmoke(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.api, cls.is_client = ida_check.open_database(SAMPLE)

    @classmethod
    def tearDownClass(cls) -> None:
        ida_check.close_database(cls.api, save=False)

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

    def test_code_without_function_gets_created_on_demand(self) -> None:
        address = next(iter(self.api.idautils.Functions()))
        function = self.api.ida_funcs.get_func(address)
        start = int(function.start_ea)
        self.assertTrue(self.api.ida_bytes.is_code(self.api.ida_bytes.get_flags(start)))
        self.assertTrue(self.api.ida_funcs.del_func(start))
        try:
            self.assertIsNone(self.api.ida_funcs.get_func(start))
            label = {
                "addr": f"{start:08x}",
                "status": "accepted",
                "record": {
                    "name": "W109OnDemandFunction",
                    "confidence": "confirmed",
                    "kind": "function",
                    "subsystem": "unknown.smoke",
                    "does": "Synthetic label used only to exercise on-demand function creation.",
                    "purpose": None,
                    "evidence": [],
                    "wire": [],
                },
            }
            map_function = {"bytes": 0, "callers": "[]", "callees": "[]", "strings": "[]"}
            row = ida_check.inspect_label(self.api, label, map_function)
            self.assertTrue(row["ida"]["function_created_on_demand"])
            self.assertTrue(row["agreement"]["boundary_match"])
        finally:
            if self.api.ida_funcs.get_func(start) is None:
                self.api.ida_funcs.add_func(start)


if __name__ == "__main__":
    unittest.main(verbosity=2)
