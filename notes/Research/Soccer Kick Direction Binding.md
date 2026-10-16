# Soccer Kick Direction Binding

Read-only bounded follow-on, 2026-10-09 (UTC). Records the verified dataflow behind the
`UpdatePlayerState` (sub 14) kick/pass direction fields and what remains unresolved. No source,
tests, content, save, UI, process or release change; all raw bytes stay in the lab. This note
deliberately does **not** name any vector by plausibility.

Owned artifacts:

| Path | Role |
| --- | --- |
| this note | verified sender/reader chain, vector provenance, unresolved list |
| `lab/reports/soccer-kick-direction-20261009T165816Z/check-soccer-kick-direction.py` | checker, sha256 `f8cb6daa95f53f645866764488a94e67d1fa7a519654d7ec533909393ff22fa6` |
| `lab/reports/soccer-kick-direction-20261009T165816Z/proof.json` | pinned proof, sha256 `c9d1904a5ddeeeb4fb6c88b5ac9eee356498b3642a86110159decfcc2cbcf935` |
| `lab/reports/soccer-kick-direction-20261009T165816Z/check-run.log` | run output, sha256 `9340494dea2f9a75b2554cd3bb8a4d5530b661791c0a6f12a874a51db57e440c` |
| `lab/reports/soccer-kick-direction-20261009T165816Z/check-sub13-offset-units.py` | correction checker, sha256 `0914d169a7ca5856b0778661306b8850f8ce492a848fe68f0f95c229a5d573f9` |
| `lab/reports/soccer-kick-direction-20261009T165816Z/correction-receipt.json` | offset-unit correction receipt, sha256 `689ec1e25d7f2da9f2614e800332f0242d86a2a4fcbe1982c4cd3db8d478f710` |
| `lab/reports/soccer-kick-direction-20261009T165816Z/correction-run.log` | correction run output, sha256 `2827e16d4b95d379778b63cb5b0b8c71f52952ce2748190f333eac307b23b988` |

Sources (hashed by the checker): `lab/decompiled/client/FreeRealms/pseudocode.c`
`f84a58674b8e7de0829d562e5f106a368f34d4750894a14b093ae745fa0befe3`,
`lab/working/OSFR.Client/OSFR Client/FreeRealms.exe`
`26cc1c522a45e71fb57882da515f65bac45c8c8b8c7e740c0cea81c52a733351`,
`lab/reports/soccer-20261009T0203Z/soccer_payloads.json`
`abf8010ffe74c6965ae4c6917de3e1b4fbdf86c1f61b42312bd83f77a2fb4fc6`.

## Verified chain

`FUN_00cdc5c0` is the sub-14 sender: SoccerPlayer vftable `0x0183A0FC` slot `+0x0C` (resolved from
the pinned exe) and the ctor is `FUN_00b746b0` (`FUN_00b730a0(0xe)` +
`SoccerPacketUpdatePlayerState::vftable`). It fires only when `player+0x2A8 != 0` and byte
`player+0xC0 != 0`, saves the previous state to `player+0x9C`, then fills the packet.

Payload offsets are fixed by the reader `FUN_00b794b0`, which consumes fields strictly in order:
`u64 guid -> u8 state -> u8 aux -> u32 word -> f32[4] -> f32[4] -> f32[4]` (62 bytes). The
sender's stack locals map onto the packet object as `local_70 = +0x00`, `local_60 = +0x10`,
`local_5c = +0x14`, `local_58 = +0x18`, `local_57 = +0x19`, `local_54 = +0x1C`,
`uStack_50..44 = +0x20..+0x2C`, `local_40..34 = +0x30..+0x3C`, `iStack_30..24 = +0x40..+0x4C`;
the reader's object offsets (`+0x10` guid, `+0x18` state, `+0x19` aux, `+0x1C` word,
`+0x20`/`+0x30`/`+0x40` vectors) corroborate that layout.

| Wire | Type | Source in `FUN_00cdc5c0` |
| --- | --- | --- |
| `0x00` | `u64` | player `+0x10`/`+0x14` |
| `0x08` | `u8` | player `+0x98` (int `0x26`) |
| `0x09` | `u8` | `(u8)((float)player+0xA4 * DAT_017ed9d0)` (int `0x29`) |
| `0x0A` | `u32` | player `+0xB0` (int `0x2c`) |
| `0x0E` | `f32[4]` | getter `vtable+0x30` -> `FUN_00cd8080` -> player `+0xF0`/`+0xF4`/`+0xF8`/`+0xFC` |
| `0x1E` | `f32[4]` | player `+0x30`..`+0x3C` (int `0x0c`..`0x0f`) |
| `0x2E` | `f32[4]` | player `+0x20`..`+0x2C` (int `0x08`..`0x0b`) |

The three vectors stay **provenance-typed, not semantically named**. What is established is only
which object field each wire block comes from; no recovered name, unit or role exists for
`player+0xF0..0xFC`, `player+0x30..0x3C` or `player+0x20..0x2C` in this pass.

## Direction and state machine anchors

* `FUN_01072020` (SoccerPlayer vftable `+0x98`) is the state setter: it stores the new state at
  `player+0x98` and then calls vtable `+0x0C` - every state change, including the kick/pass codes
  (16-26/31-34 kick family, 27-30 pass family per the reviewed rev 3 note), emits a sub-14 packet.
* `FUN_01074b50` (charge/kick machine) writes `player+0xF0..0xFC` (int `0x3c..0x3f`) from the
  `+0x30` and `+0x38` getters and calls vtable `+0x78` with that block:
  `(**(code **)(*param_1 + 0x78))(param_1 + 0x3c,1);`. It selects states `0x16`/`0x17`
  (SuperKick/SuperKickBack) and `0x23`/`0x24` (StealTackle/SlideTackleStart) around those calls.
* vtable `+0x78` resolves to `FUN_00cd8f50`, which calls `FUN_01074590(dir, flag)`; that function
  checks the squared length against `DAT_017e795c` and then calls vtable `+0x7C` with the same
  vector. The final consumer's semantics were not recovered.
* `FUN_00cde9b0` (`SoccerBall` vftable `+0x18`, sub-13 `ImpulseBall`) builds the 36-byte packet
  when its send flag is set. **Wire offsets (payload-relative, total 0x24 = 36 bytes)**: `u32 0` at
  `0x00`, `float4 param_2` at `0x04`, `ball+0x74` at `0x14`, `param_4` at `0x18`, `param_5` at
  `0x1C`, `ball+0x3C` at `0x20`. The matching **packet-object offsets** are `+0x0C`, `+0x10..+0x1F`,
  `+0x20`, `+0x24`, `+0x28`, `+0x2C` (wire = object − 0x0C), pinned by reader `FUN_00b79580` and
  the writer's stack locals (`uStack_44`, `uStack_40..34`, `uStack_30`, `uStack_2c`, `uStack_28`,
  `uStack_24`). The located dispatch in this build passes want-to-send = 0 (rev 3), and 0 of 2978
  captured records are sub 13, so no observed ball impulse exists.

## Correction receipt (2026-10-09)

Review caught a unit mix-up in the first draft of the impulse bullet: it mixed wire offsets
`0x00`/`0x04` with packet-object offsets `+0x20`/`+0x24`/`+0x28`/`+0x2C`, which sum to 0x30 (48),
not the proven 36-byte payload. The correction comes from the native bodies, not inference:
reader `FUN_00b79580` consumes `4 + 16 + 4 + 4 + 4 + 4 = 36` bytes into object `+0x0C`, `+0x10`,
`+0x20`, `+0x24`, `+0x28`, `+0x2C`, so wire = object − 0x0C and the wire offsets are `0x00`,
`0x04`, `0x14`, `0x18`, `0x1C`, `0x20`. The writer `FUN_00cde9b0` stack locals
(`uStack_44`, `uStack_40..34`, `uStack_30`, `uStack_2c`, `uStack_28`, `uStack_24`) confirm the same
object offsets, and the ctor `FUN_00b73790` (`FUN_00b730a0(0xd)` +
`SoccerPacketImpulseBall::vftable`) fixes the class.

Receipt: `lab/reports/soccer-kick-direction-20261009T165816Z/correction-receipt.json` sha256
`689ec1e25d7f2da9f2614e800332f0242d86a2a4fcbe1982c4cd3db8d478f710`; checker
`check-sub13-offset-units.py` sha256 `0914d169a7ca5856b0778661306b8850f8ce492a848fe68f0f95c229a5d573f9`;
log `correction-run.log` sha256 `2827e16d4b95d379778b63cb5b0b8c71f52952ce2748190f333eac307b23b988`.
The prior `proof.json` (`c9d1904a…`) and `check-run.log` (`9340494d…`) are preserved byte-identical;
no sub-14 claim changed. The three sub-14 vectors stay **provenance-typed only** - they are
direction-positioned fields on the wire, and their semantics remain unresolved and are not asserted.

## Unresolved (explicit)

* Semantic identity of the three source fields (`+0xF0..0xFC`, `+0x30..0x3C`, `+0x20..0x2C`);
  only provenance is proven. Not "aim", "velocity", "position" or "facing" in this note.
* The `+0x38` getter body was not decompiled this pass, and whether the live object overrides
  vtable `+0x30` is not established (the base implementation `FUN_00cd8080` only reads
  `+0xF0..0xFC`).
* Native writers of `player+0x20..0x2C` and `player+0x30..0x3C` were not located in this bounded
  pass; the input path `FUN_00afb4a0` is known only from prior notes.
* No action-id -> kick/pass state binding, no subsystem `+0x7C` consumer, and no captured
  kick/pass direction correlation was performed (flow 1 of the corpus carries no kick/pass states).
* Sub-13 stays server-authored in the located evidence; every impulse vector in tests remains
  synthetic until a capture or an independent consumer proves one.

## Authority

Sub-14 is client-authored input: the guid, state byte, aux byte and all three vectors are claims.
A server must reject `guid != authenticated sender`, accept the state only for that member, and
must not derive ball motion or trust a direction from this packet. The existing production slice
(`soccer_match::apply(sender, now, intent)`) already refuses a mismatched sender; integration must
route this decode through that guard.

## Checks

| Check | Result |
| --- | --- |
| `& '<lab>/tools/wiki-env/Scripts/python.exe' check-soccer-kick-direction.py` | exit 0; `{"proof_sha256": "c9d1904a...", "vtable_slots": {"+0x0c": "0xcdc5c0", "+0x30": "0xcd8080", "+0x78": "0xcd8f50", "+0x98": "0x1072020"}}` |
| Anchors re-read from pseudocode | sender (23 lines), reader order (8 lines), ctor/getter/`+0x78`/state setter/kick machine/impulse assertions all pass |
| Exe vtable slots | `+0x0C -> 0x00CDC5C0`, `+0x30 -> 0x00CD8080`, `+0x78 -> 0x00CD8F50`, `+0x98 -> 0x01072020` |
| `write_exact` idempotency | `proof.json` byte-identical across the logged run |
| `check-sub13-offset-units.py` | exit 0; `{"payload_bytes": 36, "prior_proof_sha256": "c9d1904a...", "wire_offsets": ["0x00", "0x04", "0x14", "0x18", "0x1c", "0x20"]}`; receipt byte-identical on re-run |

Cross-links: [[Research/Soccer Kick Pass Wire Mapping]] (rev 3),
[[Research/Soccer Native Wire Repair]], [[Research/Soccer Match Authority]].

## Follow-on: direction semantics (2026-10-09, second pass)

Appended section; everything above is preserved. New owned report:
`lab/reports/soccer-kick-direction-followon2-20261009T173220Z/` with checker
`check-soccer-direction-semantics.py` sha256 `1716a3509d7745c706c055f9573f794ec4663ea32b426ce9c3cbb685edffb11c`,
receipt `receipt.json` sha256 `f6370d3ab57a7ddaba171b16757173bb42f94d901801784eb5fa408879dd9153`,
log `check-run.log` sha256 `b2eebffb1942588b598753a73ed356285d33bb104db5d044ebe2b3e91b32347d`.
All statements below are native dataflow anchors from the pinned pseudocode/exe, not naming by
plausibility; the vectors keep their wire names.

### Resolved this pass

* **vecA (wire `0x0E`) - `player+0xF0..+0xFC`, the action-direction field.** `FUN_01074b50` copies
  the `+0x38` getter result or the `+0x30` getter/fallback vector into `+0xF0..+0xFC`; the field is
  then consumed by vtable `+0x7C` = `FUN_01072610`, which normalizes it (guard
  `DAT_017e795c` = 0.1) and writes `player+0x50..+0x5C`. The receive path stores an incoming vecA
  into the remote player's `+0xF0..+0xFC`, so the field round-trips on the wire.
* **Kick/back decision.** `FUN_01074b50` dot-products `player+0x50..+0x5C` (facing, written by
  `+0x7C`) with the `+0xF0` direction and compares against `DAT_018144d0` = **-0.2**:
  `dot >= -0.2` selects state `0x16` (`SuperKick`), otherwise the back variant `0x17`.
* **vecB (wire `0x1E`) - `player+0x30..+0x3C`, the steering/movement vector.** Its squared norm
  against `DAT_017e795c` (0.1) selects `0x23` (`StealTackle`) versus `0x24` (`SlideTackleStart`),
  and the branch copies it into `+0xF0`. Writers found: `FUN_010748a0` (per-axis conditional
  masks), `FUN_010727d0` (copies `+0xF0` components into `+0x30..` during slide progression), and
  the `FUN_01074b50` fallback `param_1[0xc] = param_1[0x14]` when the vector is near zero. The
  receive path hands vecB to `FUN_00cd97d0` (consumer not yet read).
* **vecC (wire `0x2E`) - `player+0x20..+0x2C`, consumed as the player's horizontal coordinates.**
  `FUN_00cd8330` (vtable `+0x48`) computes `(+0x20,+0x28)` deltas between two players and returns 1
  when `distance^2 <= DAT_017e99a8` (**3.0**); `FUN_00cd9df0` subtracts `+0x20`/`+0x28` from a
  target transform and zeroes the y slot. Receive path stores an incoming vecC at remote
  `+0x70..+0x7C`.
* **aux byte (wire `0x09`) = quantized charge.** `+0xA4` is written by the kicker as a charge ratio
  clamped by `DAT_0175b420` = 1.0 and read by the `FUN_00cd9af0` state hook; the sender scales it by
  `DAT_017ed9d0` = **255.0** into the byte, and the receive path scales the byte back by
  `DAT_017f1ae4` = **1/255** into `+0xA4`.
* **word (wire `0x0A`) = `player+0xB0` state-change counter.** `FUN_01072020` increments it on every
  state change (`param_1[0x2c] = param_1[0x2c] + 1`) and the receive path copies the incoming u32
  verbatim into `+0xB0`.
* **getter `+0x38` (`FUN_00cd9df0`) is target-directed**: it scans the roster for another entity
  whose `+0x1C == 3` and different `+0x18`, reads that entity's transform via `FUN_00b75660`, forms
  the horizontal delta from self `+0x20`/`+0x28`, and normalizes via `FUN_00afb6b0`; when no target
  is found it returns the constant `(1, 0, 0, 1)`.
* **Receive mapping anchors** (`FUN_00b80ac0` case `0xe`): state `->` `piVar9[0x27]`, word `->`
  `piVar9[0x2c] = local_2c4`, aux `->` `piVar9[0x29] = ... * DAT_017f1ae4`, vecA `->`
  `piVar9[0x3c..0x3f] = local_2c0..2b4`, vecC `->` `piVar9[0x1c..0x1f] = local_2a0..294`.

### Resolved vtables (exe bytes)

| Slot | Target | Role |
| --- | --- | --- |
| `+0x38` | `0x00CD9DF0` | target-directed vector getter (`FUN_00cd9df0`) |
| `+0x48` | `0x00CD8330` | proximity check using `+0x20`/`+0x28` (`FUN_00cd8330`) |
| `+0x70` | `0x00CD9AF0` | state-change hook (charge/state switch, animation slots) |
| `+0x7C` | `0x01072610` | direction consumer writing `player+0x50..+0x5C` |

### Still unresolved

* `FUN_00cd97d0` consumer semantics for the received vecB.
* Role of remote `+0x70..+0x7C` (vecC's receive target) versus local `+0x20..+0x2C`.
* The writer that maintains local `+0x20`/`+0x28` between state changes was not located in this
  bounded pass.
* The target selector `+0x1C == 3` field is unnamed, and the vecB writer set beyond the three
  functions above is not fully enumerated.

Authority is unchanged by this section: every field here is client-authored input, the direction is
usable only after sender authentication, server time and position/speed corroboration, and none of
these vectors authorizes ball motion on its own.
