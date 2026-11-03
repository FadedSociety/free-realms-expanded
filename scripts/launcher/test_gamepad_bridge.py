"""Mapping checks for gamepad_bridge.py; no controller or game needed."""

import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
SPEC = importlib.util.spec_from_file_location("gamepad_bridge", HERE / "gamepad_bridge.py")
bridge = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(bridge)
CFG = json.loads((HERE / "gamepad.json").read_text(encoding="utf-8"))
A, B = bridge.BUTTONS["A"], bridge.BUTTONS["B"]


def pad(buttons=0, lt=0, rt=0, lx=0, ly=0, rx=0, ry=0):
    return (buttons, lt, rt, lx, ly, rx, ry)


class Mapping(unittest.TestCase):
    def test_every_configured_key_name_resolves(self):
        for context in ["Default", *CFG["contexts"]]:
            cfg = bridge.layout(CFG, context)
            for key in [*cfg["buttons"].values(), *cfg["left_stick"].values()]:
                if key != "MOUSE_LEFT":
                    bridge.vk(key)
            for name in [*cfg["buttons"], cfg["cursor_mode"]]:
                bridge.held(name, 0, 0, 0)

    def test_idle_pad_holds_nothing(self):
        self.assertEqual(bridge.plan(pad(lx=3000, ry=-3000), CFG), (set(), set(), (0.0, 0.0)))

    def test_left_stick_walks_with_the_configured_keys(self):
        keys, _, _ = bridge.plan(pad(lx=-32768, ly=32767), CFG)
        stick = CFG["left_stick"]
        self.assertEqual(keys, {bridge.vk(stick["up"]), bridge.vk(stick["left"])})

    def test_buttons_and_triggers_press_their_keys(self):
        keys, mouse, _ = bridge.plan(pad(buttons=A | bridge.BUTTONS["LB"], rt=255), CFG)
        self.assertEqual(keys, {0x20, 0x09, ord("4")})
        self.assertEqual(mouse, set())

    def test_right_stick_drags_the_camera_with_the_right_button(self):
        keys, mouse, (dx, dy) = bridge.plan(pad(rx=32767, ry=32767), CFG)
        self.assertEqual((keys, mouse), (set(), {"right"}))
        self.assertAlmostEqual(dx, CFG["camera_speed"])
        self.assertAlmostEqual(dy, -CFG["camera_speed"])  # stick up moves the view up

    def test_cursor_mode_points_and_clicks_instead(self):
        keys, mouse, (dx, _) = bridge.plan(pad(buttons=A | B, lt=255, rx=32767), CFG)
        self.assertEqual((keys, mouse), (set(), {"left", "right"}))
        self.assertAlmostEqual(dx, CFG["cursor_speed"])

    def test_soccer_kicks_with_a_and_has_no_camera(self):
        soccer = bridge.layout(CFG, "Soccer")
        keys, mouse, velocity = bridge.plan(pad(buttons=A, ly=32767, rx=32767), soccer)
        self.assertEqual((keys, mouse, velocity), ({0x20, 0x26}, set(), (0.0, 0.0)))
        # Y is unbound there; the base layout's other buttons still work.
        keys, _, _ = bridge.plan(pad(buttons=bridge.BUTTONS["Y"] | bridge.BUTTONS["START"]), soccer)
        self.assertEqual(keys, {0x1B})

    def test_fishing_casts_with_the_left_mouse_and_keeps_the_camera(self):
        _, mouse, (dx, _) = bridge.plan(pad(buttons=bridge.BUTTONS["X"], rx=32767), bridge.layout(CFG, "Fishing"))
        self.assertEqual(mouse, {"left", "right"})
        self.assertAlmostEqual(dx, CFG["camera_speed"])

    def test_arrows_and_delete_are_sent_as_extended_keys(self):
        self.assertEqual(bridge.key_input(0x26, False).ki.dwFlags, 0x0001)
        self.assertEqual(bridge.key_input(0x26, True).ki.dwFlags, 0x0003)
        self.assertEqual(bridge.key_input(bridge.vk("DELETE"), False).ki.dwFlags, 0x0001)
        self.assertEqual(bridge.key_input(0x20, True).ki.dwFlags, 0x0002)

    def test_every_zone_names_a_layout(self):
        self.assertLessEqual(set(CFG["zones"].values()), set(CFG["contexts"]))

    def test_the_client_log_picks_the_layout(self):
        with tempfile.TemporaryDirectory() as folder:
            log = Path(folder) / "FreeRealms.log"
            log.write_text("old session: sg_soccer.map\nMiniGame:JoinGame\n")
            watch = bridge.LogWatch(log, CFG["zones"])
            self.assertEqual(watch.poll(), "Default")  # what was there before is not replayed

            def write(*lines, raw=""):
                with log.open("a") as out:
                    out.write("".join(f"2026-11-03\t12:59:51\t\t{line}\n" for line in lines) + raw)
                return watch.poll()

            self.assertEqual(write("AssetRequested sg_soccer_0_0.gcnk"), "Default")
            self.assertEqual(write("MiniGame:JoinGame"), "Soccer")
            self.assertEqual(write("MiniGame:LeaveMiniGame"), "Default")
            log.write_text("")  # a new client run starts a new log
            self.assertEqual(write("asset not in manifest: sg_dm_arena.map", "MiniGame:JoinGame"), "Vehicle")
            # The HUD quit logs no leave; the next join without a zone of its own is Default.
            self.assertEqual(write("MiniGame:JoinGame"), "Default")
            # A line caught half-written is read whole on the next poll.
            self.assertEqual(write("sg_fishing_medpond.gzne", raw="2026-11-03\t12:59:51\t\tMiniGame:Join"), "Default")
            self.assertEqual(write(raw="Game\n"), "Fishing")

    def test_pointer_is_kept_inside_the_window(self):
        rect = (100, 50, 1100, 850)
        self.assertEqual(bridge.keep_inside((600, 400), rect), (600, 400))
        self.assertEqual(bridge.keep_inside((5000, -20), rect), (1000, 130))
        anchor = bridge.drag_anchor(rect)
        self.assertEqual(bridge.keep_inside(anchor, rect), anchor)


if __name__ == "__main__":
    unittest.main()
