"""Attach Frida to a local test client and log labelled native hook points.

Debug harness, native side (the Lua probe covers UI scripts). Attaches only to
the FreeRealms.exe started from a given run directory, writes one JSON line per
hit to <run>/native-hook.jsonl (merged by live_log.py), and never modifies the
client's code paths: hooks read registers/memory and return.

Usage:
  python native_hook.py <run_dir> [--set quest_offer] [--list]

Hook sets are plain tables below; add an entry to watch a new address. Addresses
are FreeRealms.exe virtual addresses from the native map (image base 0x400000);
the agent rebases them against the loaded module.
"""
import argparse
import json
import sys
import time
from pathlib import Path

import frida

IMAGE_BASE = 0x400000

# kind: "enter" logs on function entry, "leave" logs the return value,
# "insn" logs registers at an instruction. `read` lists extra values to log:
#   "reg:eax", "u8:eax", "mem32:ebx+0x1c", "mem32:esp+0x70", "arg:0", "bytes:arg0:arg1:16", "bytes:arg0:64"
HOOK_SETS = {
    "quest_offer": [
        {"addr": 0x00A107F0, "kind": "enter", "label": "quest.dispatch.thunk",
         "read": ["arg:0", "arg:1", "bytes:arg0:arg1:16"]},
        {"addr": 0x00C7CD40, "kind": "enter", "label": "quest.dispatch",
         "read": ["arg:0", "arg:1", "bytes:arg0:arg1:16"]},
        {"addr": 0x00C7CE58, "kind": "insn", "label": "quest.info.unserialize.result",
         "read": ["u8:eax"]},
        {"addr": 0x00C7CF11, "kind": "insn", "label": "quest.info.unserialize.FAILED_branch", "read": []},
        {"addr": 0x00C7CE6A, "kind": "insn", "label": "quest.current_type",
         "read": ["reg:eax", "mem32:esp+0x70", "mem32:ebx+0x1c"]},
        {"addr": 0x00C7CE78, "kind": "insn", "label": "quest.offer.DROPPED_duplicate", "read": ["mem32:ebx+0x1c"]},
        {"addr": 0x00C7CEA2, "kind": "insn", "label": "quest.offer.passed_check", "read": []},
        {"addr": 0x00A94BE0, "kind": "enter", "label": "quest.ShowQuestStartScreen", "read": ["arg:0"]},
        # Reader state right after QuestInfoPacket::ReadFields returns (00c7bb60 frame):
        # [esp+0x10] error byte, [esp+8] cursor, [esp+0xc] end, [esp+0x24] allow-trailing flag.
        {"addr": 0x00C7BB9B, "kind": "insn", "label": "quest.info.reader_state",
         "read": ["mem8:esp+0x10", "mem32:esp+0x8", "mem32:esp+0xc", "mem8:esp+0x24", "mem32:esp+0x0"]},
    ],
    "dialog_camera": [
        # ReplacePointerAtECAndDispatch: switches the camera to ControllerDialog with
        # target pair (arg0, arg1). Callers: ShowDialogFromPacket (26/3),
        # ShowQuestStartScreen (49/1), ShowQuestEndScreen (49/13). The return address
        # says which one; the pair must be the NPC's GUID for the zoom to frame it.
        {"addr": 0x008D2BA0, "kind": "enter", "label": "camera.dialog_switch",
         "read": ["mem32:esp+0x0", "arg:0", "arg:1", "arg:2", "arg:3"]},
        {"addr": 0x00A94BE0, "kind": "enter", "label": "quest.ShowQuestStartScreen",
         "read": ["arg:0", "arg:1", "arg:2", "arg:3", "arg:4", "arg:5"]},
        {"addr": 0x009F5AE0, "kind": "enter", "label": "dialog.ShowDialogFromPacket",
         "read": ["arg:0", "bytes:arg0:64"]},
    ],
    "quest_end": [
        {"addr": 0x00C7CD40, "kind": "enter", "label": "quest.dispatch",
         "read": ["arg:0", "arg:1", "bytes:arg0:arg1:16"]},
        # QuestEndPacket::Unserialize (00c7bbd0) reader state after ReadFields, then result.
        {"addr": 0x00C7BC0B, "kind": "insn", "label": "quest.end.reader_state",
         "read": ["mem8:esp+0x10", "mem32:esp+0x8", "mem32:esp+0xc", "mem8:esp+0x24", "mem32:esp+0x0"]},
        {"addr": 0x00C7CF74, "kind": "insn", "label": "quest.end.unserialize.result", "read": ["u8:eax"]},
        {"addr": 0x00A95420, "kind": "enter", "label": "quest.ShowQuestEndScreen",
         "read": ["arg:0", "arg:1", "arg:2", "arg:3", "arg:4", "arg:5", "arg:6"]},
        # SendQuestEndReplyPacket sends 49/14 only when the word at this+0x1c is positive.
        {"addr": 0x00A97EF0, "kind": "enter", "label": "quest.SendQuestEndReply", "read": ["mem32:ecx+0x1c"]},
        {"addr": 0x00A864C0, "kind": "enter", "label": "quest.QuestEndReplyPacketCtor", "read": ["arg:0"]},
    ],
}

AGENT = r"""
const mod = Process.getModuleByName('FreeRealms.exe');
const IMAGE_BASE = %d;
const hooks = %s;
function at(addr) { return mod.base.add(addr - IMAGE_BASE); }
function regValue(ctx, name) { return ctx[name]; }
function readValue(spec, ctx, args) {
  try {
    const parts = spec.split(':');
    if (parts[0] === 'reg') return regValue(ctx, parts[1]).toString();
    if (parts[0] === 'u8') return regValue(ctx, parts[1]).toUInt32() & 0xff;
    if (parts[0] === 'arg') return args ? args[parseInt(parts[1])].toString() : null;
    if (parts[0] === 'mem8') {
      const m = parts[1].match(/^(\w+)\+0x([0-9a-f]+)$/i);
      return regValue(ctx, m[1]).add(parseInt(m[2], 16)).readU8();
    }
    if (parts[0] === 'mem32') {
      const m = parts[1].match(/^(\w+)\+0x([0-9a-f]+)$/i);
      return regValue(ctx, m[1]).add(parseInt(m[2], 16)).readU32();
    }
    if (parts[0] === 'bytes') {
      const p = args[parseInt(parts[1].slice(3))];
      // "bytes:argP:argN:max" (length from an argument) or "bytes:argP:len" (fixed).
      const n = parts.length === 3 ? parseInt(parts[2])
        : Math.min(args[parseInt(parts[2].slice(3))].toUInt32(), parseInt(parts[3]));
      if (p.isNull() || n <= 0) return null;
      return Array.from(new Uint8Array(p.readByteArray(n))).map(b => b.toString(16).padStart(2, '0')).join('');
    }
  } catch (e) { return 'ERR:' + e.message; }
  return null;
}
function emit(h, ctx, args, extra) {
  const values = {};
  for (const spec of h.read) values[spec] = readValue(spec, ctx, args);
  send(Object.assign({label: h.label, addr: '0x' + h.addr.toString(16), values: values}, extra || {}));
}
for (const h of hooks) {
  const target = at(h.addr);
  if (h.kind === 'insn') {
    Interceptor.attach(target, function () { emit(h, this.context, null); });
  } else {
    Interceptor.attach(target, {
      onEnter(args) { this.h = h; emit(h, this.context, args, {phase: 'enter'}); },
      onLeave(retval) { if (h.kind === 'leave') send({label: h.label, phase: 'leave', retval: retval.toString()}); }
    });
  }
}
send({label: 'native_hook.ready', values: {module_base: mod.base.toString(), hooks: hooks.length}});
"""


def find_client_pid(run):
    client = (run / "client").resolve()
    for proc in frida.get_local_device().enumerate_processes(scope="full"):
        if proc.name.lower() != "freerealms.exe":
            continue
        path = str(proc.parameters.get("path", ""))
        if path and Path(path).resolve().is_relative_to(client):
            return proc.pid
    return None


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("run_directory", type=Path)
    parser.add_argument("--set", default="quest_offer", choices=sorted(HOOK_SETS))
    parser.add_argument("--list", action="store_true", help="Print the hook set and exit")
    parser.add_argument("--wait", type=float, default=600, help="Seconds to wait for the run's client")
    args = parser.parse_args()
    hooks = HOOK_SETS[args.set]
    if args.list:
        for hook in hooks:
            print(f"0x{hook['addr']:08x} {hook['kind']:5} {hook['label']} {hook['read']}")
        return
    run = args.run_directory.resolve()
    deadline = time.monotonic() + args.wait
    pid = None
    while pid is None and time.monotonic() < deadline:
        pid = find_client_pid(run)
        if pid is None:
            time.sleep(1)
    if pid is None:
        sys.exit("native_hook: no FreeRealms.exe from this run directory")
    out = (run / "native-hook.jsonl").open("a", encoding="utf-8", buffering=1)

    def on_message(message, _data):
        record = {"event": "native_hook", "wall_ms": time.time_ns() // 1_000_000, "pid": pid}
        if message.get("type") == "send":
            record.update(message["payload"])
        else:
            record.update({"label": "native_hook.agent_error", "error": message.get("description"),
                           "stack": message.get("stack")})
        out.write(json.dumps(record) + "\n")

    session = frida.attach(pid)
    script = session.create_script(AGENT % (IMAGE_BASE, json.dumps(hooks)))
    script.on("message", on_message)
    script.load()
    detached = []
    session.on("detached", lambda reason, *_: detached.append(reason))
    try:
        while not detached:
            time.sleep(0.5)
    except KeyboardInterrupt:
        pass
    finally:
        out.close()


if __name__ == "__main__":
    main()
