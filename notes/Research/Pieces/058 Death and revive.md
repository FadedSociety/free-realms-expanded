# 058 Death and revive

Status: ready for review, 2026-10-09 05:32 UTC. This piece recovers the **knock-out and revive
wire, the client revive entry points and the native respawn-window handling**, and implements the
once-only knockout/revive policy the server needs. It is a recovered wire plus a bounded policy
module plus a checker, not a playable claim and not a native acceptance run.

Owned paths: `prototype/osfr-tools/src/restoration/piece_058.rs`,
`prototype/osfr-tools/tests/piece_058.rs`, `scripts/pieces/piece_058.py`,
`tests/pieces/test_piece_058.py`, this note and `lab/reports/piece-058-20261009T0532Z/`. No shared
`lib.rs`, gameplay, gateway, Cargo manifest, launcher, content, state or native UI file was touched.

## Recovered original facts

Respawn window packet. `EncounterShowRespawnWindowPacket` is the **encounter family** — outer opcode
`41` (`BaseEncounterPacket`) — with sub-opcode **125** and the shared 12-byte family header
`[i32 encounter id][i32 instance id]`. The evidence, all in
`lab/decompiled/client/FreeRealms/pseudocode.c` (verbatim excerpts in
`lab/reports/piece-058-20261009T0532Z/recovered-lines.txt`):

* `FUN_008a2fd0` (line 976599) is the family constructor: `param_1[1] = 0x29;` (outer `41`),
  `param_1[2] = param_2;` (sub-opcode) and two zeroed header words.
* `FUN_008a33b0` (line 976767) is the class constructor: it calls `FUN_008a2fd0(0x7d);`, installs
  `EncounterShowRespawnWindowPacket::vftable` and zeroes **exactly two** further `i32` fields
  (`param_1[5]`, `param_1[6]`).
* `FUN_008d6690` (line 1015470) reads the four header words (`u16` outer, `u16` sub, two `i32`);
  `FUN_008d6730` (line 1015514) reads the two trailing `i32` fields, which is the 20-byte body this
  piece encodes.
* `FUN_0090d830` (line 1065223) is the dispatcher: sub `0x7d` constructs the class and then calls
  `FUN_009797a0("GroupHandler:SetKO",0,0)` followed by `FUN_00904300(local_14,local_10)`, while sub
  `0x7e` (= 126) calls `FUN_009797a0("RespawnWindow:Hide",0,0)`.
* `FUN_00904300` (line 1057757) is the display handler: `if (param_2 < 1) param_2 =
  *(int *)(param_1 + 0x918);` substitutes a client-side default timer, `param_2 / 1000` reaches the
  label, and the handler ends in `FUN_009797a0("RespawnWindow:DisplayRespawn",local_208,0)`.
* The supplied server enum `LibSOE/SOE/BaseEncounterPackets.cs` independently carries
  `EncounterShowRespawnWindowPacket = 125`.

Client contract. `lab/decompiled/client-ui/ScriptsBase.lua` line 6430 declares
`RespawnWindow:DisplayRespawn(timer, isOverworld, reviveCost)`; it sets the wallet and cost through
`setCoinAmount_lua` / `setReviveCost_lua`, shows the coin dialog with `showTwoButtons_lua` only when
`isOverworld == 1` (the label template is `Revive here:\n %1 coins`) and otherwise calls
`StartTimer(timer)`. Its revive entry points are the `respawn` event -> `Ui.RevivePlayer()` (logged
through `Ui.SetWallOfData("RespawnWindow", "OnNonPaidRevive")`) and `OnPaidRevive(b)` ->
`Ui.OnPaidRevive(b)`; line 6471 forwards the window's events from the GFX. `GroupHandler:SetKO`
(line 24751) is the HUD knock-out state the dispatcher triggers.

Host calls. The shipped client's Lua host-function name table (`FreeRealms.exe` sha256
`26cc1c522a45e71fb57882da515f65bac45c8c8b8c7e740c0cea81c52a733351`, ASCII table at offset
21153752; excerpt in `client-host-function-names.txt`) registers `RevivePlayer` and `OnPaidRevive`
beside `IsInOverworldCombatArea`, `IsInOverworldCombat` and `IsInHub`. The revive calls are therefore
real host functions, and the paid-versus-free dialog choice is a **client-side overworld
predicate**, not a respawn-window field.

Localisation. The client locale resolves the countdown strings the window shows: 408279
`Recover Now!`, 408280 `Recover in`, 410188 `You Got Knocked Out!`; the knock-out state also carries
280 `You can't use abilities when you are knocked out!` and 37466 (`Once you're knocked out, you'll
be down for a few seconds …`). Work item 037 recovered the same window's GFX buttons and the free
"Revive at safe location: Free" label; this piece re-verifies that export
(`28991f9ac66221db`) instead of duplicating it.

## Community facts (never merged into the original section)

* Safe teleport request. `PacketZoneSafeTeleportRequest` is top-level opcode **122** with an empty
  body. Two independent community implementations answer it with
  `ClientUpdatePacketUpdateLocation` (**38/12**, `Teleport = true`) at a safe hub point of interest:
  pinned Sanctuary (`PacketZoneSafeTeleportRequest.cs` `c8200ce3db9ade8c`, its handler
  `b14fd7cef548d1e8`, the serializer `ec6ffc8e1189da07`) and the supplied OSFR gateway
  (`LoginManager.cs` `631fc8da1871bd73`). The shipped client also declares the same class
  (`FUN_00bf7a50`, `PacketZoneSafeTeleportRequest::vftable`).
* Two community name tables agree on the number and disagree about nothing that matters here: the
  Sanctuary packet-name table (`b8e568af2b5e69b6`) names family 41 sub 125 and has **no** entry for
  126, which matches the client's 126 branch that only hides the window, and the OSFR packet-code
  enum (`f3f1287671de00cd`) independently carries `EncounterShowRespawnWindowPacket = 125` beside
  `EncounterParticipantResumePacket = 122`.
* Wiki corpus (`b9d7b9f1adabb42d`, read-only): the "Combat" page (revision 10466) documents that after
  a knock-out the player waits ten seconds, recovers with a brief invulnerability and returns at
  full health and energy, and that wandering battle instances allow 10 knock-outs while dungeons
  allow 15. The "Sunstone Valley" page (revision 50424) documents the overworld choice: revive in a
  safezone, or spend 1,000 coins to revive on the spot of defeat.

## Reconstructed boundary

The module implements the recovered wire and a bounded policy on top of it. Labelled
reconstructed: the once-only [`ReviveArbiter`] (one live offer per knock-out, a replayed KO message
cannot re-open the window, a replayed revive cannot charge or heal twice, a coin revive below cost
changes nothing) and the [`TerminalLedger`] that keeps one terminal verdict per encounter instance
so replaying terminal messages cannot mint a reward. Health, wallet, revive cost, timer and the safe
destination are caller content; the module never invents a balance value. The free revive is the
community-documented safe-location path, and the module emits the pinned 38/12 teleport for it.

The **field order of the two trailing `i32` values is inferred, not captured**: the native handler
uses its first argument as a millisecond timer and its second as the revive cost, so the module
writes `[timer_ms][revive_cost]` and says so in the module and in the checker output.

## Explicit gaps

* No retail capture of `41/125` or `41/126` exists in `lab/reports/original-minigame-packets/`
  (50 encounter-family frames were scanned; they cover sub-codes 2, 106, 107, 108, 109, 114, 120,
  121 and 132, none of them 125 or 126). The packet body is proven by the client class fields and
  the native readers, not by a frame. A future capture makes the checker fail with `CapturePresent`
  so the gap is rewritten rather than quietly left stale.
* The client request that carries the **paid** revive is unrecovered.
* The writer that sets the client's overworld byte (`client+0x80`) is not recovered; only the
  client-side predicates that read that state are named.
* The recovery timer, the revive cost and the safe-location choice are server content; no value is
  invented here.

## Verification

```
& .\scripts\cargo.ps1 test --manifest-path prototype\osfr-tools\Cargo.toml --test piece_058
  RED  module absent: error: couldn't find file `tests\..\src\restoration\piece_058.rs`
  GREEN: test result: ok. 14 passed; 0 failed; 0 ignored
& '..\lab\tools\wiki-env\Scripts\python.exe' -m unittest discover -s tests/pieces -p "test_piece_058.py" -v
  RED  checker absent: Ran 9 tests, FAILED (failures=9)
  GREEN: Ran 11 tests, OK
& '..\lab\tools\wiki-env\Scripts\python.exe' scripts\pieces\piece_058.py --output <report>\piece-058-evidence.json
  piece 058 evidence ok: encounter 41/125 window with 20 body bytes, hide sub 126, 50 captured
  encounter-family frames
rustfmt --check --edition 2024 <both owned Rust files>
  clean
```

Evidence holds the RED/GREEN logs, the checker output (`piece-058-evidence.json`), the verbatim
client/server excerpts (`recovered-lines.txt`) and the host-function name-table excerpt
(`client-host-function-names.txt`). Source hashes (first 16 hex digits, full values in the evidence
JSON): client pseudocode `f84a58674b8e7de0`, client Lua `c7c71c7cbb177c97`, revive-window GFX
`28991f9ac66221db`, `BaseEncounterPackets.cs` `390c4802599e1dba`, Sanctuary packet table
`b8e568af2b5e69b6`, safe-teleport request `c8200ce3db9ade8c`, location serializer
`ec6ffc8e1189da07`, safe-teleport handler `b14fd7cef548d1e8`, OSFR `LoginManager.cs`
`631fc8da1871bd73`, OSFR packet-code enum `f3f1287671de00cd`, wiki `b9d7b9f1adabb42d`, locale
`f1f22a30107c824a`, client EXE `26cc1c522a45e71f`. Every hash is recomputed by
`scripts/pieces/piece_058.py`.

## Next gates

1. Lead integration: register `pub mod piece_058;` under `src/restoration/` and drop the `#[path]`
   include from `tests/piece_058.rs`.
2. Encounter integration: `src/encounter.rs` currently reports a knock-out with a chat line and an
   immediate checkpoint return. Wiring in `ReviveArbiter` + `RespawnWindow` is what turns that into
   the recovered 41/125 window, the 41/126 hide, the paid/free choice and the 38/12 relocation. The
   `gameplay.rs` relocation guard already refuses developer teleport while knocked out.
3. Gateway integration: accept the client's bare top-level 122 (`decode_safe_teleport_request`) and
   answer with the safe-location revive; the paid request packet is still missing.
4. Native acceptance (lead-owned): knock out in the overworld, confirm the two-button dialog and
   the free safe relocate, then knock out inside an encounter and confirm the countdown path; claim
   nothing from this piece until that run is recorded.

Cross-links: [[../Design/MMO Worker Contract]], [[../../MMO Work Queue]], [[037 Medic progression]].
