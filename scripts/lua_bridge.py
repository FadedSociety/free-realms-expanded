"""Drive the JSON bridge (scripts/lua_bridge.lua) in a live native test client.

The bridge runs inside the client's UI Lua and reads <run>/lua-bridge-commands.jsonl;
it answers with the same id in <run>/lua-bridge-state.jsonl. The client runs in
<run>/client, which is why the bridge paths start with "../". Nothing here talks
to the game server: the bridge calls the client's own UI actions
(Ui.ProcessChatCommand, Ui.DoClosestInteraction, DialogHandler:SelectOption and
the radial menus' click path), so a command is exactly a click by name.

Usage:
  python scripts/lua_bridge.py <run> state
  python scripts/lua_bridge.py <run> click dialog --text "Goodbye"
  python scripts/lua_bridge.py <run> click radial --text "Talk"
  python scripts/lua_bridge.py <run> chat "realm goto samantha"
  python scripts/lua_bridge.py <run> interact
  python scripts/lua_bridge.py <run> demo

The run must have been launched with run-local-baseline.py --lua-probe; the
demo writes its transcript to <run>/lua-bridge-demo.jsonl.
"""
from __future__ import annotations

import argparse
import json
import os
import sys
import time
from pathlib import Path

COMMANDS = "lua-bridge-commands.jsonl"
RESULTS = "lua-bridge-state.jsonl"
DEMO = "lua-bridge-demo.jsonl"


class BridgeError(RuntimeError):
    """The bridge did not answer, or the run directory is not usable."""


class Bridge:
    """One run's command writer and result reader."""

    def __init__(self, run, timeout: float = 10.0, poll: float = 0.05):
        self.run = Path(run).resolve()
        if not self.run.is_dir():
            raise BridgeError(f"run directory not found: {self.run}")
        self.timeout = float(timeout)
        self.poll = float(poll)
        self.commands_path = self.run / COMMANDS
        self.results_path = self.run / RESULTS
        self._sequence = 0
        self._prefix = f"{os.getpid()}-{time.time_ns()}"
        self._commands = self.commands_path.open("a", encoding="utf-8", newline="\n")
        self._offset = 0
        self._pending = b""
        self._results: dict[str, dict] = {}

    def close(self):
        self._commands.close()

    def __enter__(self):
        return self

    def __exit__(self, *_exc):
        self.close()

    def _next_id(self) -> str:
        self._sequence += 1
        return f"{self._prefix}-{self._sequence}"

    def send(self, cmd: str, **fields) -> str:
        """Append one command object and return its id (results are matched by id)."""
        if "id" in fields or "cmd" in fields:
            raise ValueError("the bridge owns the id and cmd fields")
        record = {"id": self._next_id(), "cmd": cmd}
        record.update(fields)
        self._commands.write(json.dumps(record, ensure_ascii=False, separators=(",", ":")) + "\n")
        self._commands.flush()
        return record["id"]

    def _drain(self) -> None:
        try:
            size = self.results_path.stat().st_size
        except FileNotFoundError:
            return
        if size < self._offset:
            # The results file was truncated; start over rather than read from the middle.
            self._offset = 0
            self._pending = b""
            self._results.clear()
        if size == self._offset:
            return
        with self.results_path.open("rb") as handle:
            handle.seek(self._offset)
            data = handle.read(size - self._offset)
        self._offset = size
        data = self._pending + data
        lines = data.split(b"\n")
        self._pending = lines.pop()
        for raw in lines:
            if not raw.strip():
                continue
            try:
                record = json.loads(raw.decode("utf-8"))
            except (UnicodeDecodeError, json.JSONDecodeError):
                continue
            if isinstance(record, dict) and record.get("id") is not None:
                self._results[str(record["id"])] = record

    def wait(self, request_id: str, timeout: float | None = None) -> dict:
        limit = self.timeout if timeout is None else float(timeout)
        deadline = time.monotonic() + limit
        while True:
            self._drain()
            result = self._results.get(str(request_id))
            if result is not None:
                return result
            if time.monotonic() >= deadline:
                raise BridgeError(
                    f"no result for {request_id} within {limit:g}s; "
                    f"is the client running with --lua-probe? results: {self.results_path}")
            time.sleep(self.poll)

    def request(self, cmd: str, timeout: float | None = None, **fields) -> dict:
        return self.wait(self.send(cmd, **fields), timeout=timeout)

    def state(self, timeout: float | None = None) -> dict:
        return self.request("state", timeout=timeout)

    def click(self, target: str, *, text: str | None = None, index: int | None = None,
              timeout: float | None = None) -> dict:
        if (text is None) == (index is None):
            raise ValueError("click needs exactly one of text or index")
        fields: dict = {"target": target}
        if text is not None:
            fields["text"] = text
        else:
            fields["index"] = int(index)
        return self.request("click", timeout=timeout, **fields)

    def chat(self, text: str, timeout: float | None = None) -> dict:
        if not isinstance(text, str) or not text:
            raise ValueError("chat needs a non-empty string")
        return self.request("chat", timeout=timeout, text=text)

    def interact(self, timeout: float | None = None) -> dict:
        return self.request("interact", timeout=timeout)


def _rows(state: dict, menu: str) -> list[dict]:
    return list((state.get(menu) or {}).get("buttons") or [])


def find_label(rows: list[dict], wanted: str) -> dict:
    """Case-insensitive exact label, then a unique case-insensitive substring."""
    folded = wanted.casefold()
    for row in rows:
        if str(row.get("text", "")).casefold() == folded:
            return row
    partial = [row for row in rows if folded in str(row.get("text", "")).casefold()]
    if len(partial) == 1:
        return partial[0]
    labels = ", ".join(repr(str(row.get("text", ""))) for row in rows[:8]) or "none"
    if partial:
        raise BridgeError(f"{wanted!r} is ambiguous; rows: {labels}")
    raise BridgeError(f"{wanted!r} is not offered; rows: {labels}")


def choose_page(rows: list[dict]) -> dict:
    """The first dialog row that is not the Goodbye row."""
    for row in rows:
        if str(row.get("text", "")).casefold() != "goodbye":
            return row
    raise BridgeError("the dialog offers no page row")


def _wait_state(bridge: Bridge, predicate, timeout: float, description: str) -> dict:
    deadline = time.monotonic() + timeout
    last: dict = {}
    while True:
        last = bridge.state()
        if predicate(last):
            return last
        if time.monotonic() >= deadline:
            raise BridgeError(f"timed out waiting for {description}; last state keys: "
                              f"{', '.join(sorted(last)) or 'none'}")
        time.sleep(0.25)


def demo(run, wait: float = 20.0, bridge_timeout: float = 10.0,
         page: str | None = None) -> list[dict]:
    """Record the native path: in world, goto an NPC, Talk by name, open a page."""
    bridge = Bridge(run, timeout=bridge_timeout)
    transcript: list[dict] = []
    transcript_path = bridge.run / DEMO

    def step(name: str, request: dict, result: dict) -> None:
        record = {"step": name, "request": request, "result": result}
        transcript.append(record)
        line = json.dumps(record, ensure_ascii=False)
        print(line)
        with transcript_path.open("a", encoding="utf-8", newline="\n") as handle:
            handle.write(line + "\n")

    try:
        step("state.initial", {"cmd": "state"}, bridge.state())
        world = _wait_state(bridge, lambda state: bool(state.get("position")), wait,
                            "an in-world player position")
        step("state.world", {"cmd": "state"}, world)
        goto = bridge.chat("realm goto samantha")
        step("chat.goto", {"cmd": "chat", "text": "realm goto samantha"}, goto)
        near = _wait_state(
            bridge,
            lambda state: "samantha" in str(state.get("closest", "")).casefold(),
            wait, "Samantha as the closest interaction target")
        step("state.near", {"cmd": "state"}, near)
        step("interact", {"cmd": "interact"}, bridge.interact())
        menu = _wait_state(
            bridge,
            lambda state: (state.get("radial") or {}).get("shown") and _rows(state, "radial"),
            wait, "the NPC interaction menu")
        step("state.menu", {"cmd": "state"}, menu)
        talk = find_label(_rows(menu, "radial"), "talk")
        step("click.talk", {"cmd": "click", "target": "radial", "text": talk["text"]},
             bridge.click("radial", text=talk["text"]))
        dialog = _wait_state(
            bridge,
            lambda state: (state.get("dialog") or {}).get("shown")
            and (state.get("dialog") or {}).get("rows"),
            wait, "the NPC dialog rows")
        step("state.dialog", {"cmd": "state"}, dialog)
        row = find_label(dialog["dialog"]["rows"], page) if page else choose_page(
            dialog["dialog"]["rows"])
        previous = (dialog.get("dialog") or {}).get("text")
        step("click.page", {"cmd": "click", "target": "dialog", "text": row["text"]},
             bridge.click("dialog", text=row["text"]))
        shown = _wait_state(
            bridge,
            lambda state: (state.get("dialog") or {}).get("shown")
            and (state.get("dialog") or {}).get("text") not in (None, previous),
            wait, "the page body")
        step("state.page", {"cmd": "state"}, shown)
        return transcript
    finally:
        bridge.close()


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("run", type=Path, help="existing client-baseline run directory")
    parser.add_argument("--timeout", type=float, default=10.0,
                        help="seconds to wait for one bridge answer (default 10)")
    sub = parser.add_subparsers(dest="command", required=True)
    sub.add_parser("state")
    click = sub.add_parser("click")
    click.add_argument("target", choices=("dialog", "radial", "housing"))
    click.add_argument("--text")
    click.add_argument("--index", type=int)
    chat = sub.add_parser("chat")
    chat.add_argument("text")
    sub.add_parser("interact")
    demo_parser = sub.add_parser("demo")
    demo_parser.add_argument("--wait", type=float, default=20.0,
                             help="seconds to wait for each visible step (default 20)")
    demo_parser.add_argument("--page", help="page row label; default: first row that is not Goodbye")
    args = parser.parse_args(argv)

    try:
        if args.command == "demo":
            transcript = demo(args.run, wait=args.wait, bridge_timeout=args.timeout, page=args.page)
            print(json.dumps({"demo": str(Path(args.run).resolve() / DEMO),
                              "steps": len(transcript)}, ensure_ascii=False))
            return 0
        with Bridge(args.run, timeout=args.timeout) as bridge:
            if args.command == "state":
                result = bridge.state()
            elif args.command == "click":
                result = bridge.click(args.target, text=args.text, index=args.index)
            elif args.command == "chat":
                result = bridge.chat(args.text)
            else:
                result = bridge.interact()
    except (BridgeError, ValueError, OSError) as error:
        print(f"lua_bridge: {error}", file=sys.stderr)
        return 2
    print(json.dumps(result, ensure_ascii=False))
    return 0 if result.get("ok") else 1


if __name__ == "__main__":
    raise SystemExit(main())
