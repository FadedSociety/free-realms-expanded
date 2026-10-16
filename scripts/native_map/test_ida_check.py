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
                          "strings": {"required": [], "missing": []},
                          "string_evidence": {"required": [], "missing": []}},
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

    def test_review_queue_uses_expanded_evidence_for_required_string_flags(self) -> None:
        row = self.row()
        row["agreement"]["strings"] = {"required": ["wide text"], "missing": ["wide text"]}
        row["agreement"]["string_evidence"] = {"required": ["wide text"], "missing": []}
        self.assertIsNone(ida_check.review_queue_entry(row))

        row["agreement"]["string_evidence"]["missing"] = ["wide text"]
        entry = ida_check.review_queue_entry(row)
        self.assertEqual(entry["severity"], "medium")
        self.assertEqual(entry["reasons"], [{"code": "required_strings_missing", "strings": ["wide text"]}])

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
        self.assertEqual(summary["string_method"], "IDA data-reference targets that IDA recognizes as C strings")
        self.assertIn("string_evidence_method", summary)


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


class Test00EnhancedStringEvidence(unittest.TestCase):
    class FakeIda:
        def __init__(self, functions=None, data_refs=None, code_refs=None, strings=None, words=None, calls=None):
            self.functions = functions or {}
            self.data_refs = data_refs or {}
            self.code_refs = code_refs or {}
            self.strings = strings or {}
            self.words = words or {}
            self.calls = set(calls or [])
            mapped = set(self.words)
            mapped.update(address for address in self.strings)

            class Insn:
                ea = 0

            def decode(insn, address):
                insn.ea = address
                return 1

            self.ida_bytes = SimpleNamespace(
                get_strlit_contents=self.get_strlit_contents,
                get_flags=lambda address: 1 if address in mapped else 0,
                is_data=lambda flags: flags == 1,
                is_mapped=lambda address: address in mapped,
                get_dword=lambda address: self.words.get(address, 0),
            )
            self.ida_nalt = SimpleNamespace(
                STRTYPE_C=0,
                STRTYPE_C_16=1,
                get_str_type=lambda address: self.strings.get(address, (-1, None))[0],
            )
            self.ida_funcs = SimpleNamespace(
                get_func=lambda address: next((
                    SimpleNamespace(start_ea=start, end_ea=start + 8)
                    for start in self.functions if start <= address < start + 8
                ), None),
            )
            self.idautils = SimpleNamespace(
                FuncItems=lambda start: self.functions.get(start, []),
                DataRefsFrom=lambda address: self.data_refs.get(address, []),
                CodeRefsFrom=lambda address, _flow: self.code_refs.get(address, []),
                CodeRefsTo=lambda _address, _flow: [],
            )
            self.ida_ua = SimpleNamespace(insn_t=Insn, decode_insn=decode)
            self.ida_idp = SimpleNamespace(is_call_insn=lambda insn: insn.ea in self.calls)
            self.ida_name = SimpleNamespace(get_name=lambda _address: "")
            self.imagebase = 0x400000
            self.hexrays_available = False

        def get_strlit_contents(self, address, _length, kind):
            expected, value = self.strings.get(address, (-1, None))
            if expected != kind:
                return None
            codec = "utf-16le" if kind == 1 else "utf-8"
            return value.encode(codec) + (b"\0\0" if kind == 1 else b"\0")

    def test_utf16_literal_is_recorded_as_wide_evidence(self) -> None:
        api = self.FakeIda(
            functions={0x401000: [0x401000]},
            data_refs={0x401000: [0x500000]},
            strings={0x500000: (1, "Wide text")},
        )
        evidence = ida_check._reachable_string_evidence(api, 0x401000)
        self.assertTrue(any(
            row["value"] == "Wide text" and row["encoding"] == "utf-16le" and row["method"] == "direct"
            for row in evidence
        ))

    def test_simple_pointer_table_records_string_slot_and_target(self) -> None:
        api = self.FakeIda(
            functions={0x401000: [0x401000]},
            data_refs={0x401000: [0x510000]},
            strings={0x520000: (0, "table text")},
            words={0x510000: 0x520000, 0x510004: 0},
        )
        evidence = ida_check._reachable_string_evidence(api, 0x401000)
        self.assertTrue(any(
            row["value"] == "table text" and row["method"] == "table_pointer"
            and row["table"] == "00510000" and row["slot"] == "00510000"
            and row["target"] == "00520000"
            for row in evidence
        ))

    def test_only_one_direct_callee_level_is_followed(self) -> None:
        api = self.FakeIda(
            functions={0x401000: [0x401000], 0x402000: [0x402000], 0x403000: [0x403000]},
            data_refs={0x402000: [0x520000], 0x403000: [0x530000]},
            code_refs={0x401000: [0x402000], 0x402000: [0x403000]},
            strings={0x520000: (0, "callee text"), 0x530000: (0, "grandchild text")},
            calls=[0x401000, 0x402000],
        )
        evidence = ida_check._reachable_string_evidence(api, 0x401000)
        callee = next(row for row in evidence if row["value"] == "callee text")
        self.assertEqual(callee["via"]["function"], "00402000")
        self.assertFalse(any(row["value"] == "grandchild text" for row in evidence))

    def test_new_evidence_resolves_flag_without_changing_direct_string_fields(self) -> None:
        api = self.FakeIda(
            functions={0x401000: [0x401000]},
            data_refs={0x401000: [0x500000]},
            strings={0x500000: (1, "Wide text")},
        )
        label = {"addr": "00401000", "status": "accepted", "record": {
            "name": "WideStringUser", "evidence": [{"t": "string", "v": "Wide text"}],
        }}
        row = ida_check.inspect_label(api, label, {"bytes": 8, "callers": "[]", "callees": "[]", "strings": "[]"})
        self.assertEqual(row["ida"]["referenced_strings"], [])
        self.assertEqual(row["agreement"]["strings"]["missing"], ["Wide text"])
        self.assertTrue(row["agreement"]["string_evidence"]["present"])
        self.assertNotIn("label_string_evidence_missing", [flag["code"] for flag in row["flags"]])


@unittest.skipUnless(os.environ.get("OSFR_RUN_IDA_TESTS") == "1", "IDA-backed fixture tests require OSFR_RUN_IDA_TESTS=1")
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

    def test_enhanced_string_evidence_on_small_pe_fixture(self) -> None:
        evidence = []
        for address in self.api.idautils.Functions():
            evidence = ida_check._reachable_string_evidence(self.api, address)
            if evidence:
                break
        self.assertTrue(evidence, "where.exe should expose a deterministic string reference")
        self.assertTrue(all(row.get("value") and row.get("encoding") for row in evidence))


if __name__ == "__main__":
    unittest.main(verbosity=2)
