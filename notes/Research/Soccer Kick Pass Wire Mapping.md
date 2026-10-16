# Soccer Kick Pass Wire Mapping

## Scope

Recover the original native client's soccer kick/pass wire mapping: the exact
constructor, field reader, dispatch and client send site for the family-76
intent packets, and decide whether kick/pass travel in the `UpdatePlayerControls`
control bits or in their own packet. Read-only work: decompiled client
pseudocode, the preserved `FreeRealms.exe`, the preserved soccer capture corpus
and the pinned community sources. No source, lib, gateway, Cargo, save, release,
UI, native-window or process change; no game traffic.

## Verdict

> **Revisions 2 and 3 (below) refine this verdict**: the client's kick/pass
> *intent* travels in sub 14 `UpdatePlayerState` as a named state enum, and no
> client author path for sub 13 was located in this decompiled listing - the
> single located `SoccerBall vftable+0x18` dispatch passes want-to-send = 0, so
> its impulse direction is an inference until a capture shows a client-authored
> sub 13. Revision 3 also records the bounded search limits. The rest of this
> section remains accurate for the sub-13 payload itself.

Kick/pass is a **separate packet**: `SoccerPacketImpulseBall`, family 76 sub 13,
with its own constructor (`FUN_00b73790`), field reader (`FUN_00b79580` behind
`FUN_00b7b840`), dispatch case (0xd in `FUN_00b80ac0`) and client send/apply site
(`FUN_00cde9b0`, `SoccerBall::vftable + 0x18`). Its payload is 36 bytes:

```
u32 | f32[4] | f32 | f32 | u32 | f32
```

The sub-16 `UpdatePlayerControls` packet is **not** the kick/pass carrier: its
fourth dword is an integer control bitfield assembled from input actions, and
its first three payload fields are a player guid plus a two-float direction.
Every captured sub-16 value of that dword (0, 1, 2, 8, 0x11, 0x41, 0x100) is an
integer; reading it as a float would make it zero or a denormal. No evidence
connects any individual control bit to kick or pass.

The sub-13 payload carries **no player guid and no session id**. The first
payload word is written as 0 by the client send path. A server therefore has to
bind an impulse to the authenticated session and the current possession, not to
a field in the packet.

## Evidence

| Source | Path | sha256 |
| --- | --- | --- |
| Decompiled native client | `lab/decompiled/client/FreeRealms/pseudocode.c` | `f84a58674b8e7de0829d562e5f106a368f34d4750894a14b093ae745fa0befe3` |
| Preserved client binary | `lab/working/OSFR.Client/OSFR Client/FreeRealms.exe` | `26cc1c522a45e71fb57882da515f65bac45c8c8b8c7e740c0cea81c52a733351` |
| Soccer capture corpus (2978 records) | `lab/reports/soccer-20261009T0203Z/soccer_payloads.json` | `abf8010ffe74c6965ae4c6917de3e1b4fbdf86c1f61b42312bd83f77a2fb4fc6` |
| Packet id enum (original server build) | `lab/decompiled/server/LibSOE/SOE/BaseSoccerPackets.cs` | `8aea4e73031a0b0ed89f148e46f0375da0381fa577ea83fb37f84c8e0219c304` |
| Community name table | `lab/references/Sanctuary/src/Sanctuary.Packet.Common/Extensions/PacketReaderExtensions.cs:657-684` | `b8e568af2b5e69b64b4c2642b9a788ebe2bcb1d6444ff2811e9d182a436fdb75` |
| Pinned emulator opcode (`BaseSoccerPacket = 76`) | `lab/references/OpenSourceFreeRealms/ofrserver/Server/LibSOE/Core/PacketCodes.cs` | `f3f1287671de00cd009dd86ceaeec4765be581cd823d1da3306910361ccd0869` |
| Client Lua action enum | `lab/decompiled/client-ui/ScriptsBase.lua:3035-3045` | `c7c71c7cbb177c975a7cdb12ed0c2e1b74d662f5ca5ef6a4e9ff86d2d332a8f6` |

Community status: `Sanctuary`, `FreeRealms-Legacy` and `ofrserver` pin the
family id and the sub-id **names** only. None of them contains a sub-13 payload
serializer, field list or sample; the name table is the only community input
used here.

## Recovered payloads

All soccer packets share one inner header read by `FUN_00b77fc0` (two `u16`:
`0x004C` family and the sub id, object `+0x04`/`+0x08`, wire bytes 0-3). Payload
offsets below are relative to the byte after that header; native object offsets
start at `+0x0C`.

### Sub 13 `SoccerPacketImpulseBall` - 36 bytes

| Wire offset | Type | Native field | Client send value | Class |
| --- | --- | --- | --- | --- |
| 0x00 | `u32` | `+0x0C` | `0` | Original |
| 0x04 | `f32[4]` | `+0x10..+0x1F` | `param_2[0..3]` (float4 argument) | Original |
| 0x14 | `f32` | `+0x20` | `SoccerBall + 0x74` | Original width; label inference |
| 0x18 | `f32` | `+0x24` | `param_4` (float argument) | Original |
| 0x1C | `u32` | `+0x28` | `param_5` (integer switch argument) | Original |
| 0x20 | `f32` | `+0x2C` | `SoccerBall + 0x3C` | Original width; label inference |

Reader order and widths: `FUN_00b79580` reads `+0x0C` (4 bytes), then
`FUN_008e2410` reads four 4-byte values into `+0x10..+0x1F`, then `+0x20`,
`+0x24` (both with float NaN checks), then `+0x28` (no NaN check), then `+0x2C`
(NaN check). The send site fills the same offsets in the same order, including
the literal `uStack_44 = 0;` for `+0x0C` and `uStack_28 = param_5;` for the
integer field.

### Sub 16 `SoccerPacketUpdatePlayerControls` - 20 bytes

| Wire offset | Type | Native field | Client send value | Class |
| --- | --- | --- | --- | --- |
| 0x00 | `u64` | `+0x10`/`+0x14` | player guid (instance in the low dword, slot mask in the high dword) | Original |
| 0x08 | `f32` | `+0x18` | direction x | Original |
| 0x0C | `f32` | `+0x1C` | direction y | Original |
| 0x10 | `u32` | `+0x20` | control bits | Original |

`FUN_00b793c0` reads the two guid words, two NaN-checked floats and then `+0x20`
without a float check. `FUN_00cdc810` (SoccerPlayer vftable `+0xA0`) takes
`(float, float, int)` and stores the int at `+0x20`. `FUN_00afb4a0` assembles
the bits from active input action ids (`|1`, `|2`, `|8`, `|0x10`) and calls that
slot. The corpus agrees: 1216 sub-16 records, all 20 bytes, 8 player guids, and
the last dword only ever holds `0/1/2/8/17/65/256`.

## Constructor, reader and dispatch map

| Role | Address | Pseudocode lines | Exe file offsets |
| --- | --- | --- | --- |
| Base family-76 ctor: `+0x04 = 0x4C`, `+0x08 = sub` | `FUN_00b730a0` | 1543556-1566 | `0x7730a0` (`param_1[1] = 0x4c;`) |
| Sub 13 ctor: `FUN_00b730a0(0xd)` + `SoccerPacketImpulseBall::vftable` | `FUN_00b73790` | 1543822-1839 | `0x773790`; vftable imm `0x018232F4` at `+0x39` |
| Sub 16 ctor: `FUN_00b730a0(0x10)` + `SoccerPacketUpdatePlayerControls::vftable` | `FUN_00b74920` | 1544599-4616 | `0x774920`; vftable imm `0x01823354` at `+0x39` |
| Base inner-header reader (two `u16`) | `FUN_00b77fc0` | 1547287-1547310 | `0x777fc0` |
| Sub 13 field reader | `FUN_00b79580` | 1548538-1548598 | `0x779580` |
| Sub 13 read wrapper (bounds + error flag) | `FUN_00b7b840` | 1550188-1550213 | `0x77b840` |
| Sub 16 field reader | `FUN_00b793c0` | 1548428-1548481 | `0x7793c0` |
| Sub 16 read wrapper | `FUN_00b7b760` | 1550132-1550157 | `0x77b760` |
| Family-76 dispatch (`SoccerProcessor` vtable `+0x10`); `case 0xd` constructs + reads + applies, `case 0x10` reads controls | `FUN_00b80ac0` | 1553971-... (case 0xd 1555239-1555270, case 0x10 1555367-1555413) | `0x780ac0` |
| Sub 13 apply/send (`SoccerBall` vtable `+0x18`) | `FUN_00cde9b0` | 1822995-1823058 | `0xcde9b0` |
| Sub 16 send (`SoccerPlayer` vtable `+0xA0`) | `FUN_00cdc810` | 1821466-1821515 | `0xcdc810` |
| Controller input -> control bits -> player slot `+0xA0` | `FUN_00afb4a0` | 1451985-1452077 | `0xafb4a0` |

Verified vtables in the exe (COL -> RTTI name, then the slot):

| Class | vftable VA | slot | entry |
| --- | --- | --- | --- |
| `SoccerBall` | `0x0183A46C` (COL `0x0195CB9C` -> `.?AVSoccerBall@@`) | `+0x18` | `0x00CDE9B0` |
| `SoccerBallBase` | `0x0188378C` (COL `0x01970C94` -> `.?AVSoccerBallBase@@`) | `+0x18` | `0x0107BCE0` (overridden by `SoccerBall`) |
| `SoccerPlayer` | `0x0183A0FC` (COL `0x0195CAE8` -> `.?AVSoccerPlayer@@`) | `+0xA0` | `0x00CDC810` |
| `SoccerProcessor` | `0x0182399C` (COL `0x0194BE9C` -> `.?AVSoccerProcessor@@`) | `+0x10` | `0x00B80AC0` |

`lab/reports/soccer-wire-20261009T114735Z/` holds the machine-generated report,
logs and fixtures for all of the above.

## Original, inference, missing

Original (byte-exact, independently checkable):

* Family id `0x4C`, sub ids 13/16, the 36-byte and 20-byte payload layouts and
  the unsigned/float typing of every field, from the readers plus the matching
  send-site fills.
* The two packet vftable addresses embedded in the constructors and the four
  class vtables above, from raw exe bytes and MSVC RTTI.
* The sub-16 control dword is an integer bitfield (reader, sender, corpus).
* Sub 13 is absent from the capture (0 of 2978 records), so no captured vector
  exists; nothing here is a capture claim.

Inference (labelled, not authoritative):

* Sub 13 field labels beyond `u32/f32/f32/u32`: `+0x20` is loaded from
  `SoccerBall + 0x74`, which `SoccerBallBase + 0x18` (`FUN_0107bce0`) clamps as
  a bounded scalar and divides by a parameter into `+0x78`; the `SoccerBall`
  constructor also registers `bbe_scr_ball_charge.xml`,
  `bbe_scr_ball_charged_01.xml` and `bbe_ball_superkick_trail.xml`. That is
  consistent with a charge/power-shaped field, but the packet never names it.
* The `param_5` switch in the send path groups values 0/2/3 against
  `SoccerBall + 0x144` and value 1 against `+0x14C`; the switch target is an
  action-shaped slot, but the mapping from the Lua actions
  (`Kick = 2`, `Pass = 6`) into this integer is **not** proven.
* `u32` and `u64` here are unsigned; no signed field appears in either layout.

Missing (do not invent):

* Any captured sub 13 vector, so no original impulse was ever observed.
* The native action-to-ImpulseBall binding: `SoccerPlayerActionsDataSource`
  exists (RTTI descriptor at file `0x177A8FC`), and Lua names Kick/Pass, but no
  decompiled call site maps an action id into `FUN_00cde9b0`'s `param_5`.
* `FUN_00cde9b0`'s only virtual dispatch in the whole listing is the inbound
  `case 0xd` call at line 1555263, which passes `SoccerBall vftable + 0x18`.
  The send branch (`param_6 != 0`, gated on a live gateway at
  `*(*(param_1 + 0x120) + 0x280) != 0`) therefore has no located caller in this
  build; who sends an impulse, and with what arguments, stays unproven.
* Field semantics for `+0x0C`, the float4 and the final float.

## Authority implications

* A sub-13 impulse names no player, so an implementation must bind it to the
  authenticated session, the current holder and the server tick; the packet
  alone cannot authorize a kick.
* The sub-16 control bits are advisory input state, not the kick/pass intent;
  a server that treats them as kick/pass would invent semantics the client
  never sends.
* Impulse values are attacker-controlled floats; a server slice must bound the
  resulting ball velocity and ignore impulses from players without possession,
  and must not trust the claimed ball charge.
* The existing reconstructed `src/soccer_match.rs` kick/pass intents remain
  server-side policy: this mapping does not make them historical, and the
  native kick binding stays unproven until the lead tests it.

## Checks

All sources are read-only lab inputs.

| Check | Command | Result | Log |
| --- | --- | --- | --- |
| Checker (16 pseudocode anchors, exe constructors, 2978-record corpus) | `& '..\lab\tools\wiki-env\Scripts\python.exe' scripts/pieces/soccer_wire.py --output <report>` | `soccer wire evidence ok: 16 pseudocode anchors, sub 16 1216/2978 records, sub 13 captured 0, control bits [0, 1, 2, 8, 17, 65, 256]` | `lab/reports/soccer-wire-20261009T114735Z/checker.log` |
| Unit tests (synthetic layout/tamper + real-source checks) | `& '..\lab\tools\wiki-env\Scripts\python.exe' -m unittest discover -s tests/pieces -p "test_soccer_wire.py" -v` | `Ran 18 tests ... OK` | `lab/reports/soccer-wire-20261009T114735Z/tests.log` |
| Captured sub-16 decode example | `... soccer_wire.py --sub 16 --decode <captured payload>` | `player 274886297266, x -1.0, y 1.0, bits 65` | `lab/reports/soccer-wire-20261009T114735Z/decode-example.json` |

The checker fails closed if a pseudocode anchor leaves its owning function, if
either constructor stops embedding its vftable, or if any captured sub-16 record
stops looking like an integer bitfield. Tests use synthetic vectors only and
label them as such.

## Artifacts

| Artifact | sha256 |
| --- | --- |
| `scripts/pieces/soccer_wire.py` | see `summary.json` |
| `tests/pieces/test_soccer_wire.py` | see `summary.json` |
| `lab/reports/soccer-wire-20261009T114735Z/report.json` | checker report (anchors + exe + corpus) |
| `lab/reports/soccer-wire-20261009T114735Z/tests.log` | 18 passed |
| `lab/reports/soccer-wire-20261009T114735Z/summary.json` | machine-readable summary of this note |

## Limitations and next acceptance

* The send branch of `FUN_00cde9b0` is the only ImpulseBall writer found; its
  caller inside the client was not located, so client kick binding is unproven.
  Next step: find the caller that passes `param_6 != 0` (search for the
  `SoccerBall` vtable `+0x18` dispatch outside line 1555263) or capture sub 13
  on the wire.
* The semantic names of the sub-13 fields are inference; a single captured or
  live sub-13 impulse would settle them.
* No native acceptance, no client run, no gateway integration and no packet was
  sent by this work.

## Revision 2 - the state channel (2026-10-09, later same day)

Revision 1 of this note is preserved above and is pinned at sha256
`04f8fac85a6c4c66e3bc845966fb68a80c35a3c073d507044ccf89f21a21ae79`. Two of
its statements are refined here: the exact sub-14 layout, and where the client's
kick/pass action actually goes on the wire.

### Sub 13 author-path search (locally bounded; corrected in Revision 3)

`FUN_00cde9b0` (`SoccerBall::vftable + 0x18`) builds and sends
`SoccerPacketImpulseBall` only inside `if (param_6 != '\0')`. The only dispatch
of that slot in the client is the family-76 `case 0xd` apply path, and its raw
bytes show the last stack argument is the immediate 0:

```
0x00B82871  e8 ca 14 ff ff   call FUN_00b73d40        ; get the ball
            ...
            8b 16            mov edx, [esi]           ; vtable
            8b 52 18         mov edx, [edx+0x18]      ; slot 6
            6a 00            push 0                   ; want-to-send = 0
            50               push eax
            83 ec 08         sub esp, 8
            d9 5c 24 04      fstp dword [esp+4]
            d9 1c 24         fstp dword [esp]
            51               push ecx                 ; &vec4
            8b ce            mov ecx, esi
            ff d2            call edx
```

`check_want_to_send_zero` re-reads this site and asserts the three markers in
order (`8b 52 18`, `6a 00`, `ff d2`). Qualified consequence (Revision 3): **no
client author path for sub 13 was located in this decompiled listing**. The one
located dispatch passes want-to-send = 0, so this site applies the impulse
locally without echoing it. Absence of every send path is not established - the
decompiled listing has zero textual callers of `FUN_00cde9b0` (every invocation
is vtable-mediated) and the image holds 122 untyped generic slot-0x18 load
sites that were not receiver-typed. The direction of travel remains an
inference until a capture shows a client-authored sub 13.

### Sub 14 UpdatePlayerState is the client's action channel - 62 bytes

Reader `FUN_00b794b0` (wrapper `FUN_00b7b7d0`, dispatch `case 0xe`), sender
`FUN_00cdc5c0` (`SoccerPlayer::vftable + 0x0C`, slot 3):

| Wire offset | Type | Native field | Client send value | Class |
| --- | --- | --- | --- | --- |
| 0x00 | `u64` | `+0x10`/`+0x14` | player guid | Original |
| 0x08 | `u8` | `+0x18` | **soccer player state code** (`player+0x98`) | Original |
| 0x09 | `u8` | `+0x19` | `(int)(float(player+0xA4) * const)` (aux byte) | Original |
| 0x0A | `u32` | `+0x1C` | `player+0xB0` | Original |
| 0x0E | `f32[4]` | `+0x20` | `vtable+0x30` call result | Original |
| 0x1E | `f32[4]` | `+0x30` | `player+0x30..0x3F` | Original |
| 0x2E | `f32[4]` | `+0x40` | `player+0x20..0x2F` | Original |

The chain is complete in code: the state setter `FUN_01072020`
(`SoccerPlayerBase::vftable + 0x98`, inherited by `SoccerPlayer`) stores
`param_1[0x26] = param_2` (object `+0x98`) and then calls
`(**(code **)(*param_1 + 0xc))();` - in the `SoccerPlayer` vftable that slot is
`FUN_00cdc5c0`, the sub-14 sender, which reads `player+0x98` into the state byte
and saves the previous state to `player+0x9C` before sending. Every state change
therefore produces an `UpdatePlayerState` message.

### State enum (client's own name table)

The client carries a 64-entry state-name pointer table at VA `0x01BA2338`
(file `0x17A2338`); index == wire state code. `check_exe_state_table` resolves
all 64 names from the preserved exe and fails if any entry changes.

| Code | Name | Code | Name |
| --- | --- | --- | --- |
| 0 | None | 17 | KickRight |
| 1 | Idle | 18 | KickBack |
| 2 | Run | 19-21 | HighKickLeft/Right/Back |
| 3 | Jump | 22 | SuperKick |
| 4 | Juke | 23 | SuperKickBack |
| 5 | JukeWithBall | 24-25 | KickLeft/RightRunning |
| 6 | Juke360WithBall | 26 | LobKickRunning |
| 7 | Land | 27 | PassKickRunning |
| 8-9 | TurnLeft/TurnRight | 28 | HighPassKickRunning |
| 10 | AcquireBall | 29-30 | PassKickBack / HighPassKickBack |
| 11 | RunWithBall | 31-32 | KickBackRunning / HighKickBackRunning |
| 12-13 | ReceiveBall / RunningReceiveBall | 33-34 | KickLeft/RightRunningStop |
| 14-15 | TurnLeft/RightWithBall | 35 | StealTackle |
| 16 | KickLeft | 36-38 | SlideTackleStart/Middle/End |
| 39-43 | Stumble, Impact, HitLow/Mid/High | 51-63 | goalie states (Block*, Catch*, IdleWithBallInHand, Throw, GoalieKick) |

The numeric mapping is proven by the setter's own remaps: request 2 (Run)
becomes 0xB (RunWithBall), 8/9 (Turn) become 0xE/0xF (turn with ball) while the
player holds the ball (`param_1[0x31] != 0`), and 1 (Idle) becomes 0x3D
(GoalieIdleWithBallInHand) when the previous state was 0x3A-0x3C (goalie
catches). In `FUN_01074b50` a speed threshold selects 0x23 (StealTackle) versus
0x24 (SlideTackleStart), a directional check selects 0x16/0x17
(SuperKick/SuperKickBack) and then calls `vtable+0x78` with the direction vector
at `+0x3C`, and `pcVar5(0x19)` sets 0x19 = KickRightRunning. `FUN_010727d0`
advances 0x25/0x26 (SlideTackleMiddle/End).

### What the capture shows

250 sub-14 records decode cleanly (62 bytes each). Per flow:

* flow 1 (captured client -> server): states `{1: 11, 2: 16, 35: 1, 36: 3,
  37: 3, 38: 3}` - Idle, Run, StealTackle and the three SlideTackle phases.
  This client never transmitted a kick or pass state in the capture.
* flow 0 (server -> client): includes `{17: 2, 22: 2}` - KickRight and
  SuperKick relayed for other players - plus goalie states 52/53/55/60-62.

So kick/pass **is** a client-to-server signal in this build, but it is the
sub-14 state byte (16-34 for kicks/passes), not the control bits and not sub 13.
Whether the server answers a kick state with a sub-13 impulse is not observable
in the preserved capture (0 of 2978 records) and stays an explicit gap.

### Lua Kick=2 / Pass=6 are tutorial columns

`BaseClient.SoccerProcessor.SoccerPlayerActions` is registered by its own ctor
`FUN_00cd7840` (called from the `SoccerProcessor` ctor at pseudocode 1553729);
the name string appears exactly once in the exe (VA `0x01839C08`) and its
pointer is referenced only by that ctor. Its only consumer is
`BaseClient_SoccerProcessor_SoccerPlayerActions_OnDataChanged` ->
`SoccerTutorial:OnDataChanged(column, value)`, where column 0/2/3/6/7/1/4/5
advance tutorial pages (ScriptsBase.lua 5045-5047, 25753-25775). The column
names in the exe near the data-source name are Ball, Dodge, SuperKick,
SlideTackle, Steal, Pass, Call, Powerup - a UI/tutorial schema, not a wire
encoding. No client code turns those Lua codes into a packet; the wire action
code is the sub-14 state enum above.

### Control bits, per flow

The only located client producer is `FUN_00afb4a0`, which maps input-action ids
to bits 1/2/8/0x10 and two direction axes before calling
`SoccerPlayer::vftable+0xA0`. The capture agrees: flow 1 carries only
`{0, 1, 2}`; the richer patterns `{8, 0x11, 0x41, 0x100}` appear only on flow 0
(server relay). No bit is named by the client, so treating any bit as kick/pass
would be invention.

### Revision 2 checks

| Check | Command | Result | Log |
| --- | --- | --- | --- |
| Checker (23 anchors, exe constructors + 5 vtable slots + state table + want-to-send bytes, 2978-record corpus) | `& '..\lab\tools\wiki-env\Scripts\python.exe' scripts/pieces/soccer_wire.py --output <report>` | `soccer wire evidence ok: 23 pseudocode anchors, sub 16 1216/2978 records, sub 14 250 records, sub 13 captured 0, control bits [0, 1, 2, 8, 17, 65, 256], kick/pass states by flow {'0': {'17': 2, '22': 2}}` | `lab/reports/soccer-wire-rev2-20261009T120723Z/checker.log` |
| Unit tests | `& '..\lab\tools\wiki-env\Scripts\python.exe' -m unittest discover -s tests/pieces -p "test_soccer_wire.py" -v` | `Ran 24 tests ... OK` | `lab/reports/soccer-wire-rev2-20261009T120723Z/tests.log` |
| Captured kick examples | `... soccer_wire.py --sub 14 --decode <captured payload>` | state 17 -> `cSoccerPlayerStateKickRight`, state 22 -> `cSoccerPlayerStateSuperKick` (direction `(0.894, 0, 0.448, 0)`, position `(242.24, -0.009, 209.34, 1.0)`), state 35 -> `cSoccerPlayerStateStealTackle` (client flow) | `lab/reports/soccer-wire-rev2-20261009T120723Z/decode-*.json` |

### Revision 2 remaining gaps

* No captured sub-13 record exists, so the server's impulse response to a kick
  state is still unproven; every sub-13 vector in the tests stays synthetic.
* The state byte's *aux* byte (`+0x19`) and the three float4s are typed and
  mapped but their gameplay meaning is not established.
* No key/mouse binding table was found in the preserved client data; the
  input-action ids seen in `FUN_00afb4a0` (0xa, 0xb, 0x10, 0x1a, 0x1b, 1-8)
  remain unnamed.

## Revision 3 - review corrections (2026-10-09)

Independent review `notes/Reviews/Soccer Kick Pass Wire Review.md`
(`046a9768...`) verified the mapping (vtable slots, constructors, state table,
sub-14 sender/setter, per-flow corpus interpretation, checker reproduction) and
qualified one claim. Revision 3 applies that qualification and closes the
documentation gaps it names. Revisions 1 and 2 stay above as research history.

### Corrected verdict

* **Not claimed any more:** "the client never authors sub 13", "sub 13 is
  server-to-client only", "the only dispatch". Those are universal negatives the
  evidence does not support.
* **Claimed now:** no client author path for sub 13 was **located** in this
  decompiled listing; the single located dispatch of `SoccerBall vftable+0x18`
  (VA `0x00B82871`) passes want-to-send = 0; the impulse direction (server to
  client) is an **inference** until a capture contains a client-authored sub 13.
* Bounded-search facts the note now states explicitly: `FUN_00cde9b0` has zero
  textual callers in `pseudocode.c` (every invocation is vtable-mediated); the
  exact dispatch string appears once; the image contains **122** generic
  `mov edx, [edx+0x18]` load sites that were not receiver-typed; the base
  `SoccerBallBase+0x18` slot resolves to a different function
  (`FUN_0107BCE0`), so base-class dispatch does not reach sub 13 either.
* `soccer_wire.py` now carries a machine-readable `SEARCH_SCOPE` block
  (`search_scope` in the checker report) whose `claim` is the bounded sentence
  and whose `not_established` field names the unproven part; the CLI prints it
  with every run, and a test asserts the wording (no "never", no
  "server-to-client only").

### Authenticated-sender rule now covers sub 14 and sub 16

Revision 2 stated the sender-binding requirement only for sub 13. It applies to
every client-authored soccer identity field, and the packets differ in what
they expose:

| Packet | Client-authored identity | Server requirement |
| --- | --- | --- |
| sub 13 `ImpulseBall` | no player field at all (field `+0x00` written as 0) | bind the impulse to the authenticated session and possession |
| sub 14 `UpdatePlayerState` | `u64 player` guid at payload 0x00 | reject `guid != authenticated sender`; never trust the state byte for another player |
| sub 16 `UpdatePlayerControls` | `u64 player` guid at payload 0x00 | reject `guid != authenticated sender`; treat bits/vectors as advisory input only |
| sub 1/3 | player guid (per the reviewed capture slice) | same rule |

The production slice already enforces this shape
(`soccer_match::apply(sender, now, intent)` refuses `sender != intent.player()`);
integration must route the decoded sub-14/sub-16 guid through that same guard
rather than trusting the field.

### Capture precision (sub 14)

251 raw sub-14 records exist in the corpus; **250** are clean 62-byte payloads.
The extra record is a zero-length flow-0 artifact,
`soccer-01284-flow0-frame808-op47-sub14.bin` (payload length 0, frame 808). The
checker now reports `player_states_raw`, `player_states` and the explicit
`player_states_skipped` list, and the test asserts all three.

### Community route divergence (acknowledged)

The pinned `Sanctuary` and `FreeRealms-Legacy` trees mention the soccer packets
only in `src/Sanctuary.Packet.Common/Extensions/PacketReaderExtensions.cs` (the
sub-id name table, line 671 for `SoccerPacketImpulseBall`); they contain no
serializer, field list or sample. There is therefore no community artifact to
reuse for the exact type/order; the mapping in this note comes from the native
decompile, and every table above says so. This is a route divergence from the
"community first" preference, recorded here rather than papered over.

### Revision 3 checks

| Check | Command | Result | Log |
| --- | --- | --- | --- |
| Checker (23 anchors, exe constructors + 5 vtable slots + state table + want-to-send bytes + bounded scope, corpus precision) | `& '..\lab\tools\wiki-env\Scripts\python.exe' scripts/pieces/soccer_wire.py --output <report>` | `... sub 14 250/251 records (1 skipped) ...; scope: no client author path for sub 13 was located in this decompiled listing (122 generic slot-0x18 sites untyped)` | `lab/reports/soccer-wire-rev3-20261009T121806Z/checker.log` |
| Unit tests | `& '..\lab\tools\wiki-env\Scripts\python.exe' -m unittest discover -s tests/pieces -p "test_soccer_wire.py" -v` | `Ran 25 tests ... OK` (adds the bounded-search wording test) | `lab/reports/soccer-wire-rev3-20261009T121806Z/tests.log` |

### Revision 3 remaining gaps (unchanged)

* No captured sub-13 vector (0 of 2978); the server's impulse response to a kick
  state is unproven and every sub-13 test vector stays labelled synthetic.
* Native kick/pass key binding is still missing: no key/mouse binding table in
  the preserved data, input-action ids in `FUN_00afb4a0` unnamed.
* Aux byte and float4 semantics in sub 14 remain typed-but-unnamed.
* Native acceptance remains with the lead; this work sent no packet and touched
  no source, lib, gateway, state or UI file.
