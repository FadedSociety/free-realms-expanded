import importlib.util
import json
import unittest
from pathlib import Path


SCRIPT = Path(__file__).resolve().parents[1] / "scripts" / "live_log.py"
SPEC = importlib.util.spec_from_file_location("live_log", SCRIPT)
live_log = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(live_log)


class LiveLogTests(unittest.TestCase):
    def test_merge_decode_filter_and_redact(self):
        npc = live_log._record(json.dumps({"event": "npc_interact", "wall_ms": 10,
                                           "player_name": "N1", "chat_text": "C1"}))
        item = {"sort": 10, "source": "server-stdout.log", "record": npc,
                "raw": "", "summary": live_log._summary("server-stdout.log", npc, ""), "severity": "info"}
        self.assertNotIn("N1", live_log._render(item))
        self.assertNotIn("C1", live_log._render(item))
        packet = live_log._record(json.dumps({"event": "packet", "wall_ms": 20, "family": 49,
                                              "subtype": 1, "family_name": "QuestInfo"}))
        item = {"sort": 20, "source": "server-trace.jsonl", "record": packet,
                "raw": "", "summary": live_log._summary("server-trace.jsonl", packet, ""), "severity": "info"}
        self.assertIn("49/1 QuestInfo", item["summary"])
        self.assertTrue(live_log._matches(item, {49}, [], "questinfo"))
        self.assertFalse(live_log._matches(item, {26}, [], ""))
        self.assertFalse(live_log._matches(item, {49}, ["questinfo"], ""))
        self.assertEqual(live_log._timestamp(npc, "", 99), 10)

    def test_compound_sensitive_fields_are_redacted(self):
        record = live_log._record(json.dumps({"gateway_ticket": "sample-a", "accountKey": "sample-b"}))
        self.assertEqual(record, {"gateway_ticket": "[redacted]", "accountKey": "[redacted]"})
        self.assertTrue(live_log._privacy_ok(record))


if __name__ == "__main__":
    unittest.main()
