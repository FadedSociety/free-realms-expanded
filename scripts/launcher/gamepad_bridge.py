"""Controller support for the Free Realms PC client.

The PC client reads only keyboard and mouse (it imports no XInput or
DirectInput), so this bridge reads an XInput pad and sends the client's own
default keys (the executable's binding table, see the vault note
Research/Native Combat Controls) plus right-mouse-drag camera.

Input is sent only while a Free Realms window is in front; everything held is
released the moment it is not. The layout is gamepad.json, re-read when it changes.

    python gamepad_bridge.py            # run until Ctrl+C
    python gamepad_bridge.py --check    # show whether a pad is connected
    python gamepad_bridge.py --pid N    # only drive client process N
"""

from __future__ import annotations

import argparse
import ctypes
import json
import sys
import time
from ctypes import wintypes
from pathlib import Path

CONFIG = Path(__file__).with_name("gamepad.json")

BUTTONS = {
    "DPAD_UP": 0x0001, "DPAD_DOWN": 0x0002, "DPAD_LEFT": 0x0004, "DPAD_RIGHT": 0x0008,
    "START": 0x0010, "BACK": 0x0020, "LTHUMB": 0x0040, "RTHUMB": 0x0080,
    "LB": 0x0100, "RB": 0x0200, "A": 0x1000, "B": 0x2000, "X": 0x4000, "Y": 0x8000,
}
NAMED_KEYS = {
    "SPACE": 0x20, "TAB": 0x09, "ESCAPE": 0x1B, "ENTER": 0x0D, "SHIFT": 0xA0,
    "CTRL": 0xA2, "ALT": 0xA4, "NUMLOCK": 0x90, "DELETE": 0x2E, "BACKSPACE": 0x08,
    **{f"F{n}": 0x6F + n for n in range(1, 13)},
}


def vk(name: str) -> int:
    name = name.upper()
    if len(name) == 1 and name.isalnum():
        return ord(name)
    return NAMED_KEYS[name]


def held(name: str, buttons: int, lt: int, rt: int) -> bool:
    if name == "LT":
        return lt > 127
    if name == "RT":
        return rt > 127
    return bool(buttons & BUTTONS[name])


def axis(value: int, deadzone: float) -> float:
    """Stick value scaled to -1..1, zero inside the deadzone."""
    v = max(-1.0, value / 32767)
    if abs(v) < deadzone:
        return 0.0
    return (abs(v) - deadzone) / (1 - deadzone) * (1 if v > 0 else -1)


def plan(pad: tuple[int, int, int, int, int, int, int], cfg: dict):
    """What should be held right now: (keys, mouse buttons, mouse velocity px/s)."""
    buttons, lt, rt, lx, ly, rx, ry = pad
    dz = cfg["deadzone"]
    keys, mouse = set(), set()
    sticks = cfg["left_stick"]
    x, y = axis(lx, dz), axis(ly, dz)
    if y > 0:
        keys.add(vk(sticks["up"]))
    if y < 0:
        keys.add(vk(sticks["down"]))
    if x < 0:
        keys.add(vk(sticks["left"]))
    if x > 0:
        keys.add(vk(sticks["right"]))

    cursor = held(cfg["cursor_mode"], buttons, lt, rt)
    for name, key in cfg["buttons"].items():
        if cursor and name in ("A", "B", cfg["cursor_mode"]):
            continue
        if held(name, buttons, lt, rt):
            keys.add(vk(key))

    cx, cy = axis(rx, dz), axis(ry, dz)
    speed = cfg["cursor_speed"] if cursor else cfg["camera_speed"]
    velocity = (cx * speed, -cy * speed * (-1 if cfg.get("invert_y") else 1))
    if cursor:
        if held("A", buttons, lt, rt):
            mouse.add("left")
        if held("B", buttons, lt, rt):
            mouse.add("right")
    elif cx or cy:
        mouse.add("right")  # the client's camera is right-mouse drag
    return keys, mouse, velocity


# ---- Windows I/O -----------------------------------------------------------

class XINPUT_GAMEPAD(ctypes.Structure):
    _fields_ = [("wButtons", wintypes.WORD), ("bLeftTrigger", ctypes.c_ubyte),
                ("bRightTrigger", ctypes.c_ubyte), ("sThumbLX", ctypes.c_short),
                ("sThumbLY", ctypes.c_short), ("sThumbRX", ctypes.c_short),
                ("sThumbRY", ctypes.c_short)]


class XINPUT_STATE(ctypes.Structure):
    _fields_ = [("dwPacketNumber", wintypes.DWORD), ("Gamepad", XINPUT_GAMEPAD)]


class MOUSEINPUT(ctypes.Structure):
    _fields_ = [("dx", wintypes.LONG), ("dy", wintypes.LONG), ("mouseData", wintypes.DWORD),
                ("dwFlags", wintypes.DWORD), ("time", wintypes.DWORD),
                ("dwExtraInfo", ctypes.c_size_t)]


class KEYBDINPUT(ctypes.Structure):
    _fields_ = [("wVk", wintypes.WORD), ("wScan", wintypes.WORD), ("dwFlags", wintypes.DWORD),
                ("time", wintypes.DWORD), ("dwExtraInfo", ctypes.c_size_t)]


class INPUT(ctypes.Structure):
    class _U(ctypes.Union):
        _fields_ = [("mi", MOUSEINPUT), ("ki", KEYBDINPUT)]
    _anonymous_ = ("u",)
    _fields_ = [("type", wintypes.DWORD), ("u", _U)]


def load_xinput():
    for name in ("xinput1_4", "xinput1_3", "xinput9_1_0"):
        try:
            return getattr(ctypes.windll, name).XInputGetState
        except OSError:
            continue
    sys.exit("No XInput DLL found on this system.")


def read_pad(get_state):
    """The first connected pad as a plan() tuple, or None."""
    state = XINPUT_STATE()
    for index in range(4):
        if get_state(index, ctypes.byref(state)) == 0:
            g = state.Gamepad
            return (g.wButtons, g.bLeftTrigger, g.bRightTrigger,
                    g.sThumbLX, g.sThumbLY, g.sThumbRX, g.sThumbRY)
    return None


user32 = ctypes.windll.user32
kernel32 = ctypes.windll.kernel32
MOUSE_FLAGS = {"left": (0x0002, 0x0004), "right": (0x0008, 0x0010)}  # (down, up)


def send(*inputs: INPUT):
    if inputs:
        user32.SendInput(len(inputs), (INPUT * len(inputs))(*inputs), ctypes.sizeof(INPUT))


def key_input(code: int, up: bool) -> INPUT:
    scan = user32.MapVirtualKeyW(code, 0)
    return INPUT(type=1, ki=KEYBDINPUT(wVk=code, wScan=scan, dwFlags=0x0002 if up else 0))


def mouse_input(flags: int, dx: int = 0, dy: int = 0) -> INPUT:
    return INPUT(type=0, mi=MOUSEINPUT(dx=dx, dy=dy, dwFlags=flags))


def client_in_front(pid: int | None) -> bool:
    window = user32.GetForegroundWindow()
    owner = wintypes.DWORD()
    user32.GetWindowThreadProcessId(window, ctypes.byref(owner))
    if pid is not None:
        return owner.value == pid
    process = kernel32.OpenProcess(0x1000, False, owner.value)  # QUERY_LIMITED_INFORMATION
    if not process:
        return False
    try:
        buffer = ctypes.create_unicode_buffer(32768)
        size = wintypes.DWORD(len(buffer))
        kernel32.QueryFullProcessImageNameW(process, 0, buffer, ctypes.byref(size))
        return Path(buffer.value).name.lower() == "freerealms.exe"
    finally:
        kernel32.CloseHandle(process)


def exited(pid: int) -> bool:
    process = kernel32.OpenProcess(0x00100000, False, pid)  # SYNCHRONIZE
    if not process:
        return True
    try:
        return kernel32.WaitForSingleObject(process, 0) == 0
    finally:
        kernel32.CloseHandle(process)


def run(pid: int | None):
    get_state = load_xinput()
    cfg, stamp = None, None
    keys, mouse = set(), set()
    carry = [0.0, 0.0]
    last = time.monotonic()
    print("Controller bridge running; Ctrl+C to stop.")
    try:
        while True:
            if CONFIG.stat().st_mtime != stamp:
                stamp = CONFIG.stat().st_mtime
                cfg = json.loads(CONFIG.read_text(encoding="utf-8"))
            now = time.monotonic()
            if pid is not None and int(now) != int(last) and exited(pid):
                return  # releases everything below
            dt, last = now - last, now
            pad = read_pad(get_state) if client_in_front(pid) else None
            want_keys, want_mouse, velocity = plan(pad, cfg) if pad else (set(), set(), (0, 0))

            send(*[key_input(k, True) for k in keys - want_keys],
                 *[key_input(k, False) for k in want_keys - keys],
                 *[mouse_input(MOUSE_FLAGS[b][1]) for b in mouse - want_mouse],
                 *[mouse_input(MOUSE_FLAGS[b][0]) for b in want_mouse - mouse])
            keys, mouse = want_keys, want_mouse

            carry = [carry[0] + velocity[0] * dt, carry[1] + velocity[1] * dt]
            step = [int(carry[0]), int(carry[1])]
            if any(step):
                send(mouse_input(0x0001, *step))
                carry = [carry[0] - step[0], carry[1] - step[1]]
            time.sleep(1 / 120)
    finally:
        send(*[key_input(k, True) for k in keys],
             *[mouse_input(MOUSE_FLAGS[b][1]) for b in mouse])


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--check", action="store_true", help="report the pad and exit")
    parser.add_argument("--pid", type=int, help="only drive this client process")
    args = parser.parse_args()
    if args.check:
        pad = read_pad(load_xinput())
        print(f"Controller connected: {pad}" if pad else "No XInput controller connected.")
        return
    try:
        run(args.pid)
    except KeyboardInterrupt:
        pass


if __name__ == "__main__":
    main()
