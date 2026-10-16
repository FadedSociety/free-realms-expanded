"""Mapping checks for gamepad_bridge.py; no controller or game needed."""

import importlib.util
import json
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
        for key in [*CFG["buttons"].values(), *CFG["left_stick"].values()]:
            bridge.vk(key)
        for name in [*CFG["buttons"], CFG["cursor_mode"]]:
            bridge.held(name, 0, 0, 0)

    def test_idle_pad_holds_nothing(self):
        self.assertEqual(bridge.plan(pad(lx=3000, ry=-3000), CFG), (set(), set(), (0.0, 0.0)))

    def test_left_stick_walks_with_the_clients_wasd(self):
        keys, _, _ = bridge.plan(pad(lx=-32768, ly=32767), CFG)
        self.assertEqual(keys, {ord("W"), ord("A")})

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


if __name__ == "__main__":
    unittest.main()
