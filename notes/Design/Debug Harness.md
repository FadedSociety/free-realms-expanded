# Debug Harness

The local gateway and login servers can write opt-in JSONL packet traces. The
baseline launcher opens a live viewer beside the client and merges those traces
with service output and client logs. Tracing is off unless `--live-log` or
`--trace FILE` is supplied.

## Start a live run

Build the local server binaries with the private toolchain from the crate directory:

```powershell
Push-Location prototype/osfr-tools
& ..\..\scripts\cargo.ps1 build --release --bin osfr-gateway --bin osfr-login
Pop-Location
```

Start the usual baseline with `--live-log`:

```powershell
& ..\lab\tools\universal-modder-env\Scripts\python.exe .\scripts\run-local-baseline.py --rust-reference <reference-dir> --content <content-dir> --live-log
```

The launcher writes `server-trace.jsonl` and, with `--native-login`,
`login-trace.jsonl` under the new `lab/runs/client-baseline-<UTC stamp>` folder.
It opens `live_log.py` in a separate console and places that console beside the
client window when Windows exposes both window rectangles. The merged stream is
also appended to `<run>/live.log`.

Run the viewer against an existing capture:

```powershell
python .\scripts\live_log.py ..\lab\runs\client-baseline-<UTC stamp>
```

The viewer reads each source from its start, then follows appended lines. Press
Ctrl+C to stop it. `--self-test` scans a recorded run without writing to it.

## Filters

```text
--only 26,49,66       Show only these game families
--hide movement,character_saved  Hide matching event, handler, or source text
--grep QuestInfo      Case-insensitive text search
```

Filters change the console view. `live.log` retains the full merged stream.
Known family/subtype pairs display labels such as `49/1 QuestInfo` and
`26/8 NpcInteract`. SOE lines show opcode, reliable sequence, and whether the
message was complete, fragmented, or combined. Red marks rejected, error, and
unhandled events; yellow marks warnings and movement clamps. Source colors
distinguish server, login, and client files.

## Sources

- `server-trace.jsonl`: gateway transport and decoded game messages, gateway
  handler outcomes, movement decisions, session events, and save/world events.
- `login-trace.jsonl`: login transport, login opcodes, and login handler results.
- `server-stdout.log` and `server-stderr.log`: existing server diagnostics,
  including `npc_*` and `quest_*` JSON events emitted by gameplay code.
- `login-stdout.log` and `login-stderr.log`: existing login service diagnostics.
- `client-stdout.log` and `client-stderr.log`: client process output.
- `client/Logs/*.log`: native logs, including `LuaErrors`, `CommandQueue`,
  `ActorFailures`, `AssetFailure`, `MissingImages`, `FreeRealms`, `NetInfo`,
  `QuestOfferLuaProbe.log`, and later probe logs.

Each packet record uses monotonic `ts_ms`; `wall_ms` lets the viewer merge
separate server and login processes on one timeline. Traces omit chat previews,
login bodies, names, keys, and tickets. The viewer redacts sensitive fields in
structured log lines before displaying or appending them to `live.log`.
Native lines with their own timestamps use those values; otherwise live mode
uses arrival time and `--self-test` uses the source file modification time.

Direct server use accepts `--trace FILE` on `osfr-gateway` and `osfr-login`.
Each trace file is created or replaced when that process starts.

## Layers (lead, 2026-10-14)

The harness has three layers. Together they show a client action from the wire to the UI.

| Layer | Option | Writes | Shows |
| --- | --- | --- | --- |
| Server trace | on by default (`--no-live-log` disables) | `server-trace.jsonl`, `login-trace.jsonl` | Every inbound message with handler and outcome (handled / ignored / rejected / UNHANDLED + reason). Every outbound message as `app_out` before SOE packing, with length, hex prefix and **cause** (the inbound messages that produced it), plus the SOE packets |
| Lua probe | `--lua-probe` | `client/Logs/QuestOfferLuaProbe.log` | Replaces the run copy's `UI/ScriptsBase.bin` with checked source plus wrappers on quest/NPC/dialog functions and UI event dispatchers |
| Native hook | `--native-hook <set>` (`native_hook.py --list`) | `native-hook.jsonl` | Frida attached to the run copy's `FreeRealms.exe`; logs labelled native addresses (entry args, registers, memory reads) |

All three feed the live viewer. The viewer never takes focus: `SetWindowPos` uses NOACTIVATE, and it is started directly with `CREATE_NEW_CONSOLE`, not `cmd start`. It checks its parent with `OpenProcess` + `GetExitCodeProcess`; never use `os.kill(pid, 0)` on Windows, which calls TerminateProcess.

**Adding a native hook:** append `{addr, kind (enter/leave/insn), label, read[]}` to a set in `project/scripts/native_hook.py`. Read specs are `reg:eax`, `u8:eax`, `mem8:esp+0x10`, `mem32:ebx+0x1c`, `arg:0` and `bytes:arg0:arg1:16`. Addresses are native-map VAs (image base 0x400000).

**First result:** the `quest_offer` set showed `QuestInfoPacket::Unserialize` returning 0 with 6 trailing bytes. Our 49/1 was 120 bytes where the client reads 114. See `Reviews/Native Round5 050310.md`.

**Install:** Frida 17.23.3 and frida-tools 14.11.0 in `lab/tools/universal-modder-env`, installed from PyPI with the owner's approval on 2026-10-14.
