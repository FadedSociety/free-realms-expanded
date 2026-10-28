"""The Lua JSON bridge and its Python driver speak one file protocol.

The bridge itself is Lua; the harness runs the real bridge in a real Lua 5.1
interpreter when one is available (set LUA to its path) and skips otherwise.
The Python driver always runs.
"""
from __future__ import annotations

import hashlib
import importlib.util
import json
import os
import shutil
import subprocess
import tempfile
import threading
import time
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
LAB = ROOT.parent / "lab"
SCRIPT = ROOT / "scripts" / "lua_bridge.py"
BRIDGE_LUA = ROOT / "scripts" / "lua_bridge.lua"
STUB = ROOT / "tests" / "fixtures" / "lua_bridge_stub.lua"


def load_script(name: str):
    spec = importlib.util.spec_from_file_location(name.replace(".py", ""), ROOT / "scripts" / name)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


lua_bridge = load_script("lua_bridge.py")
lua_probe_overlay = load_script("lua_probe_overlay.py")


def find_lua() -> str | None:
    for name in (os.environ.get("LUA"), "lua", "lua51", "lua5.1"):
        found = shutil.which(name) if name else None
        if not found:
            continue
        try:
            version = subprocess.run([found, "-v"], capture_output=True, text=True,
                                     timeout=5)
        except (OSError, subprocess.SubprocessError):
            continue
        if "Lua 5.1" in (version.stdout + version.stderr):
            return found
    return None


LUA = find_lua()


class BridgeClientTests(unittest.TestCase):
    """The Python side: ids, partial result lines, timeouts and row selection."""

    def setUp(self):
        self.temp = Path(tempfile.mkdtemp(prefix="osfr-lua-bridge-"))
        self.run = self.temp / "run"
        (self.run / "client").mkdir(parents=True)
        self.bridge = lua_bridge.Bridge(self.run, timeout=0.2, poll=0.01)

    def tearDown(self):
        self.bridge.close()
        shutil.rmtree(self.temp)

    def test_ids_are_unique_and_results_are_matched_by_id(self):
        first = self.bridge.send("state")
        second = self.bridge.send("click", target="dialog", text="Talk")
        self.assertNotEqual(first, second)
        results = self.run / lua_bridge.RESULTS
        results.write_text(
            json.dumps({"id": "another-client", "ok": True}) + "\n"
            + json.dumps({"id": first, "ok": True, "cmd": "state"}) + "\n",
            encoding="utf-8")
        self.assertEqual(self.bridge.wait(first, timeout=1)["cmd"], "state")
        with self.assertRaises(lua_bridge.BridgeError):
            self.bridge.wait(second, timeout=0.05)

    def test_a_partial_result_line_is_completed_on_the_next_read(self):
        request = self.bridge.send("state")
        results = self.run / lua_bridge.RESULTS
        whole = json.dumps({"id": request, "ok": True, "cmd": "state"})
        results.write_text(whole[:12], encoding="utf-8")
        with self.assertRaises(lua_bridge.BridgeError):
            self.bridge.wait(request, timeout=0.05)
        with results.open("a", encoding="utf-8") as handle:
            handle.write(whole[12:] + "\n")
        self.assertTrue(self.bridge.wait(request, timeout=1)["ok"])

    def test_an_unanswered_request_names_the_run_results_file(self):
        with self.assertRaisesRegex(lua_bridge.BridgeError, "lua-bridge-state.jsonl"):
            self.bridge.state()

    def test_the_bridge_owns_its_protocol_fields(self):
        with self.assertRaises(ValueError):
            self.bridge.send("state", id="forced")
        with self.assertRaises(ValueError):
            self.bridge.click("dialog")
        with self.assertRaises(ValueError):
            self.bridge.click("dialog", text="Talk", index=0)
        with self.assertRaises(ValueError):
            self.bridge.chat("")
        self.assertTrue((self.run / lua_bridge.COMMANDS).is_file())

    def test_dialog_rows_are_selected_by_name_or_first_page(self):
        rows = [{"text": "Goodbye"}, {"text": "Cooking: Strong Arm Stew"},
                {"text": "Harvesting"}]
        self.assertEqual(lua_bridge.find_label(rows, "GOODBYE")["text"], "Goodbye")
        self.assertEqual(lua_bridge.find_label(rows, "cooking")["text"],
                         "Cooking: Strong Arm Stew")
        self.assertEqual(lua_bridge.choose_page(rows)["text"], "Cooking: Strong Arm Stew")
        with self.assertRaisesRegex(lua_bridge.BridgeError, "ambiguous"):
            lua_bridge.find_label([{"text": "Talk to A"}, {"text": "Talk to B"}], "Talk")
        with self.assertRaisesRegex(lua_bridge.BridgeError, "not offered"):
            lua_bridge.find_label(rows, "Nope")
        with self.assertRaisesRegex(lua_bridge.BridgeError, "no page"):
            lua_bridge.choose_page([{"text": "Goodbye"}])


@unittest.skipUnless(LUA, "set LUA to a Lua 5.1 interpreter to run the bridge stub")
class LuaBridgeTests(unittest.TestCase):
    """The real bridge, driven through OnUpdate in a stub client."""

    def setUp(self):
        self.temp = Path(tempfile.mkdtemp(prefix="osfr-lua-bridge-lua-"))
        self.run = self.temp / "run"
        (self.run / "client").mkdir(parents=True)
        self.bridge = lua_bridge.Bridge(self.run, timeout=2)
        self.lua = LUA

    def tearDown(self):
        self.bridge.close()
        shutil.rmtree(self.temp)

    def _send(self, requests):
        return [self.bridge.send(cmd, **fields) for cmd, fields in requests]

    def _tick(self, scenario):
        result = subprocess.run(
            [self.lua, str(STUB), scenario, str(BRIDGE_LUA)],
            cwd=self.run / "client", capture_output=True, text=True, encoding="utf-8",
            errors="replace", timeout=30)
        self.assertEqual(result.returncode, 0, result.stderr)
        return [line for line in result.stdout.splitlines() if line]

    def _answers(self, ids):
        return [self.bridge.wait(request, timeout=2) for request in ids]

    def _result_lines(self):
        path = self.run / lua_bridge.RESULTS
        return [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines()
                if line.strip()]

    def test_state_reports_zone_position_target_and_windows(self):
        [request] = self._send([("state", {})])
        self.assertEqual(self._tick("base"), [])
        [state] = self._answers([request])
        self.assertTrue(state["ok"])
        self.assertEqual(state["zone"], "Cobblestone")
        self.assertEqual(state["position"], [120.5, 3.25, -44.75])
        self.assertEqual(state["closest"], "Samantha")
        self.assertIn("Main.wndInventory", state["windows"])
        self.assertNotIn("dialog", state)

    def test_dialog_rows_and_named_clicks_use_the_client_paths(self):
        requests = [
            ("state", {}),
            ("click", {"target": "dialog", "text": "cook"}),
            ("click", {"target": "dialog", "index": 2}),
            ("click", {"target": "dialog", "text": "nope"}),
            ("click", {"target": "radial", "text": "Talk"}),
            ("key", {"key": "E"}),
        ]
        ids = self._send(requests)
        self.assertEqual(self._tick("dialog"), ["dialog\t0", "dialog\t2"])
        state, first, second, missing, no_menu, key = self._answers(ids)
        self.assertEqual(state["dialog"]["text"], "Samantha greets you.")
        self.assertEqual([row["text"] for row in state["dialog"]["rows"]],
                         ["Cooking: Strong Arm Stew", "Harvesting", "Goodbye"])
        self.assertEqual((first["ok"], first["index"], first["text"]), (True, 0,
                         "Cooking: Strong Arm Stew"))
        self.assertEqual((second["ok"], second["index"]), (True, 2))
        self.assertIn("no dialog row matches", missing["error"])
        self.assertIn("radial menu is not shown", no_menu["error"])
        self.assertIn("unsupported", key["error"])

    def test_radial_and_housing_clicks_send_their_event_ids(self):
        ids = self._send([
            ("state", {}),
            ("click", {"target": "radial", "text": "talk"}),
            ("click", {"target": "radial", "text": "nope"}),
        ])
        self.assertEqual(self._tick("radial"), ["radial\t7"])
        state, clicked, refused = self._answers(ids)
        self.assertEqual(state["radial"]["buttons"],
                         [{"index": 0, "text": "Talk", "event": 7},
                          {"index": 1, "text": "Barter", "event": 8}])
        self.assertEqual((clicked["ok"], clicked["event"]), (True, 7))
        self.assertFalse(refused["ok"])

        # A fresh run for the housing radial: its click calls Housing, not Ui.
        self.bridge.close()
        shutil.rmtree(self.run)
        (self.run / "client").mkdir(parents=True)
        self.bridge = lua_bridge.Bridge(self.run, timeout=2)
        ids = self._send([("click", {"target": "housing", "text": "up"})])
        self.assertEqual(self._tick("housing"), ["housing\t1001"])
        [clicked] = self._answers(ids)
        self.assertEqual((clicked["ok"], clicked["event"]), (True, 1001))

    def test_refused_commands_run_no_client_action(self):
        ids = self._send([
            ("wat", {}),
            ("key", {"key": "E"}),
            ("move", {"x": 1}),
            ("click", {"target": "dialog", "text": "Talk"}),
            ("chat", {}),
        ])
        with (self.run / lua_bridge.COMMANDS).open("a", encoding="utf-8", newline="\n") as handle:
            handle.write("not json\n")
        self.assertEqual(self._tick("base"), [])
        for answer in self._answers(ids):
            self.assertFalse(answer["ok"], answer)
        self.assertTrue(all("error" in answer for answer in self._answers(ids)))
        self.assertIn("invalid json", self._result_lines()[-1]["error"])

    def test_utf8_text_and_json_escapes_round_trip(self):
        [request] = self._send([("chat", {"text": "caf\u00e9"})])
        with (self.run / lua_bridge.COMMANDS).open("a", encoding="utf-8", newline="\n") as handle:
            handle.write('{"id":"escape-1","cmd":"chat","text":"caf\\u00e9"}\n')
        self.assertEqual(self._tick("base"), ["chat\tcaf\u00e9", "chat\tcaf\u00e9"])
        self.assertEqual(self._answers([request])[0]["text"], "caf\u00e9")
        self.assertEqual(self.bridge.wait("escape-1", timeout=2)["text"], "caf\u00e9")

    def test_a_partial_command_line_is_read_once(self):
        (self.run / lua_bridge.COMMANDS).write_text(
            '{"id":"partial-1","cmd":"state"}', encoding="utf-8")
        self.assertEqual(self._tick("partial"), [])
        answer = self.bridge.wait("partial-1", timeout=2)
        self.assertTrue(answer["ok"])
        self.assertEqual(answer["cmd"], "state")

    def test_a_reload_does_not_repeat_an_answered_command(self):
        [request] = self._send([("click", {"target": "dialog", "text": "Goodbye"})])
        self.assertEqual(self._tick("dialog"), ["dialog\t2"])
        [answer] = self._answers([request])
        self.assertTrue(answer["ok"])
        self.assertEqual(self._tick("dialog"), [])
        self.assertEqual(sum(1 for line in self._result_lines() if line.get("id") == request), 1)

    def test_a_closed_dialog_keeps_the_last_body_for_the_driver(self):
        state_id, click_id = self._send([
            ("state", {}),
            ("click", {"target": "dialog", "text": "Goodbye"}),
        ])
        self.assertEqual(self._tick("closed"), [])
        state, refused = self._answers([state_id, click_id])
        self.assertFalse(state["dialog"]["shown"])
        self.assertEqual(state["dialog"]["text"], "The page body.")
        self.assertEqual(state["dialog"]["rows"], [{"index": 0, "text": "Goodbye"}])
        self.assertFalse(refused["ok"])
        self.assertIn("no dialog is shown", refused["error"])


class _FakeClient(threading.Thread):
    """File-protocol stand-in for a logged-in client, for the demo sequence only."""

    def __init__(self, run):
        super().__init__(daemon=True)
        self.run_dir = run
        self.stopping = threading.Event()
        self.processed = 0
        self.closest = ""
        self.radial: list[dict] = []
        self.dialog: dict | None = None
        self.chats: list[str] = []
        self.interactions = 0

    def stop(self):
        self.stopping.set()

    def _answer(self, request, **fields):
        record = {"id": request["id"], "cmd": request.get("cmd"), "ok": True}
        record.update(fields)
        with (self.run_dir / lua_bridge.RESULTS).open("a", encoding="utf-8", newline="\n") as out:
            out.write(json.dumps(record) + "\n")

    def _state(self):
        state = {"zone": "Cobblestone", "position": [0, 0, 0]}
        if self.closest:
            state["closest"] = self.closest
        if self.radial:
            state["radial"] = {"shown": True, "buttons": self.radial}
        if self.dialog:
            state["dialog"] = {"shown": True, **self.dialog}
        return state

    def _handle(self, request):
        cmd = request.get("cmd")
        if cmd == "state":
            self._answer(request, **self._state())
        elif cmd == "chat":
            self.chats.append(request.get("text"))
            if "realm goto samantha" in request.get("text", ""):
                self.closest = "Samantha"
            self._answer(request, text=request.get("text"))
        elif cmd == "interact":
            self.interactions += 1
            self.radial = [{"index": 0, "text": "Talk", "event": 7}]
            self._answer(request)
        elif cmd == "click" and request.get("target") == "radial" and self.radial:
            self.radial = []
            self.dialog = {"text": "Samantha greets you.", "rows": [
                {"index": 0, "text": "Cooking: Strong Arm Stew"},
                {"index": 1, "text": "Goodbye"}]}
            self._answer(request, target="radial", text="Talk", event=7)
        elif cmd == "click" and request.get("target") == "dialog" and self.dialog:
            self.dialog = {"text": "Stew page.", "rows": self.dialog["rows"]}
            self._answer(request, target="dialog", index=0, text=request.get("text"))
        else:
            with (self.run_dir / lua_bridge.RESULTS).open("a", encoding="utf-8",
                                                          newline="\n") as out:
                out.write(json.dumps({"id": request.get("id"), "ok": False,
                                      "error": "unexpected"}) + "\n")

    def run(self):
        commands = self.run_dir / lua_bridge.COMMANDS
        while not self.stopping.is_set():
            try:
                lines = commands.read_text(encoding="utf-8").splitlines()
            except FileNotFoundError:
                lines = []
            for line in lines[self.processed:]:
                self.processed += 1
                try:
                    record = json.loads(line)
                except json.JSONDecodeError:
                    continue
                if isinstance(record, dict):
                    self._handle(record)
            time.sleep(0.01)


class DemoTests(unittest.TestCase):
    """The recorded demo sequence over the real file protocol (client stand-in)."""

    def setUp(self):
        self.temp = Path(tempfile.mkdtemp(prefix="osfr-lua-bridge-demo-"))
        self.run = self.temp / "run"
        (self.run / "client").mkdir(parents=True)
        self.fake = _FakeClient(self.run)
        self.fake.start()

    def tearDown(self):
        self.fake.stop()
        self.fake.join(timeout=2)
        shutil.rmtree(self.temp)

    def test_demo_gotos_talks_clicks_a_page_and_reads_the_body(self):
        transcript = lua_bridge.demo(self.run, wait=2, bridge_timeout=2)
        self.assertEqual([step["step"] for step in transcript],
                         ["state.initial", "state.world", "chat.goto", "state.near",
                          "interact", "state.menu", "click.talk", "state.dialog",
                          "click.page", "state.page"])
        self.assertEqual(self.fake.chats, ["realm goto samantha"])
        self.assertEqual(self.fake.interactions, 1)
        self.assertEqual(transcript[6]["request"],
                         {"cmd": "click", "target": "radial", "text": "Talk"})
        self.assertEqual(transcript[8]["request"]["text"], "Cooking: Strong Arm Stew")
        self.assertEqual(transcript[-1]["result"]["dialog"]["text"], "Stew page.")
        recorded = (self.run / lua_bridge.DEMO).read_text(encoding="utf-8").splitlines()
        self.assertEqual(len(recorded), len(transcript))
        self.assertEqual(json.loads(recorded[-1])["result"]["dialog"]["text"], "Stew page.")


class ProbeCompositionTests(unittest.TestCase):
    """The run copy is the validated probe plus the repository bridge, checked as Lua."""

    def test_the_bridge_is_appended_after_the_probe(self):
        base = b"-- probe\nprobe.loaded\n"
        bridge = BRIDGE_LUA.read_bytes()
        source = lua_probe_overlay.compose_probe_source(base, bridge)
        self.assertTrue(source.startswith(b"-- probe\nprobe.loaded\n\n"))
        self.assertTrue(source.rstrip().endswith(bridge.rstrip()))
        self.assertIn(b"__luaBridgeInstalled", source)
        with self.assertRaises(ValueError):
            lua_probe_overlay.compose_probe_source(base, b"")
        with self.assertRaises(ValueError):
            lua_probe_overlay.compose_probe_source(base, b"one\r\ntwo\n")
        with self.assertRaises(ValueError):
            lua_probe_overlay.compose_probe_source(base, b"\xef\xbb\xbfbridge\n")

    @unittest.skipUnless(
        (LAB / "content/lua-probe/lua-probe.json").is_file()
        and (LAB / "working/OSFR.Client/OSFR Client/UI/ScriptsBase.bin").is_file(),
        "the private client and lua-probe package are not present")
    def test_validate_lua_probe_parses_the_exact_run_source(self):
        proof = lua_probe_overlay.validate_lua_probe(
            LAB / "content/lua-probe", LAB / "working/OSFR.Client/OSFR Client", LAB)
        self.assertEqual(proof["proof"]["bridge_source"], "scripts/lua_bridge.lua")
        self.assertEqual(proof["proof"]["bridge_source_sha256"],
                         hashlib.sha256(BRIDGE_LUA.read_bytes()).hexdigest().upper())
        self.assertEqual(proof["proof"]["run_source_sha256"],
                         hashlib.sha256(proof["run_source"]).hexdigest().upper())
        self.assertGreater(proof["proof"]["run_syntax_token_count"], 359927)
        self.assertTrue(proof["run_source"].rstrip().endswith(BRIDGE_LUA.read_bytes().rstrip()))


if __name__ == "__main__":
    unittest.main()
