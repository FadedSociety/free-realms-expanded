# 046 Consumables and buffs

Status: ready for review, 2026-10-09 04:35 UTC. This piece recovers the *use* request framing, the
item-bar slot presentation (icon/name/quantity/recast fields), the seven proof-recorded consumable
rows and the cached potion effects, then ships a labelled reconstruction of once-only consumption,
cooldown and effect expiry as an offline ledger. It is evidence plus a checker plus a small typed
module; it is not a playable claim and no native request was sent.

Owned paths: `prototype/osfr-tools/src/restoration/piece_046.rs`,
`prototype/osfr-tools/tests/piece_046.rs`, `scripts/pieces/piece_046.py`,
`tests/pieces/test_piece_046.py`, this note and `lab/reports/piece-046-20261009T0421Z/`. No shared
`lib.rs`, gameplay, gateway, Cargo manifest, launcher, content, state or native UI file was touched.

## Recovered original facts

Packet ids come from the decompiled original server enums, not from the community reconstruction:

| Family | Base | Sub | Source |
| --- | --- | --- | --- |
| `InventoryPacketItemActionBarAssign` | 42 | 6 | `lab/decompiled/server/LibSOE/SOE/BaseInventoryPackets.cs` |
| `InventoryPacketCheckImmediateActivationItem` | 42 | 16 | same |
| `InventoryPacketUseImmediateActivationItem` | 42 | 15 | same |
| `AbilityPacketClientRequestStartAbility` | 36 | 10 | `lab/decompiled/server/LibSOE/SOE/BaseAbilityPackets.cs` |
| `AbilityPacketFailed` | 36 | 1 | same |
| `ClientUpdatePacketUpdateActionBarSlot` | 38 | 25 | `lab/decompiled/server/LibSOE/SOE/BaseClientUpdatePackets.cs` |
| `ClientUpdatePacketAddEffectTag` / `RemoveEffectTag` | 38 | 16 / 17 | same; the consumable-to-effect-tag binding is unrecovered |

Request framing, recovered by the native constructor/writer extract in
`lab/reports/original-inventory-use-v1/proof.json` (SHA-256
`262d05bd51b55719ad93e27ddb988f7a22eebf9821ce8d1458ac1895cdacb9cf`) and matched by the pinned
community reader (Sanctuary revision `4f028dcc274f628b65ef665fd59b184f5f3d3e3e`, AGPL-3.0, read
as facts):

| Request | Layout | Bytes |
| --- | --- | --- |
| 42/6 assign consumable | `u16 main, u16 sub, i32 slot, i32 owned item instance id` | 12 |
| 36/10 activate | `u16, u16, i32 bar, i32 slot, i32 target mode` + `u64` GUID (mode 0) | 24 |
| 36/10 activate | same + `Vector4` (mode 1) | 32 |
| 42/16 check / 42/15 use | `u16, u16, i32 owned item instance id` | 8 |

Original client dock anchors (`lab/decompiled/client-ui/ScriptsBase.lua`, SHA-256
`c7c71c7cbb177c975a7cdb12ed0c2e1b74d662f5ca5ef6a4e9ff86d2d332a8f6`):
`activateItem` calls `Ui.ClickedAbility(tonumber(b) - 5, 2)` (bar 2, zero-based slot) at line
30493; the consumables equipment view (map 18) and the dock selector both call
`Ui.AssignToItemActionBar(slot, ownedItemGUID)` (lines 8824 and 30496).

The 38/25 reply is `4-byte header + 8-byte ActionBarData(bar, slot) + the 15-field 51-byte slot`.
The original server serializer `ActionBarSlot.Serialize`
(`lab/decompiled/server/Gateway/Gateway.Login/ClientPcData.cs`) and the native client reader
extract (`lab/reports/original-item-toolbar-v1/proof.json`, SHA-256
`eb1fe14305fb2e769beda8486bf8cf5414c5c24c9f617e68f64f5361fd47c24e`) agree on the fields. Wire
offsets inside the slot payload, with the recovered meaning where known:

| Offset | Field | Meaning |
| --- | --- | --- |
| 0 | `ItemEmpty` | empty slot |
| 1 | `Icon` | icon id (item definition `Icon.Id`) |
| 5 | `Unknown3` | icon tint |
| 9 | `ItemName` | name/string id |
| 13 / 17 / 21 | `Unknown5/6/7` | selector A, target selector, numeric target parameter (native reads it as `f32`) |
| 25 | `Unknown8` | mana cost |
| 29 | `Usable` | server-enabled bit, independent of any item-definition flag |
| 30 | `Unknown10` | unnamed |
| 34 | `Unknown11` | total recast time; 0 means no recast delay |
| 38 | `Unknown12` | unnamed |
| 42 | `ItemQuanity` | displayed quantity |
| 46 | `Unknown14` | dismount flag |
| 47 | `Unknown15` | cast age; `i32::MAX` is the never-cast sentinel, otherwise the client converts it to a clock |

Native client input gating: non-empty, no bits set in the recovered status mask `0x1542c6`, mana
sufficient and the slot server-enabled bit; the remaining recast is a separate click gate. 36/1
carries string id 3079, "You can't use that ability right now."

Recovered consumable rows (`lab/references/FreeRealms-Legacy/src/Resources/ClientItemDefinitions.json`,
pinned AGPL-3.0 revision `9e082025391177d5a33ee18c714ddd06b457a695`; strings resolved through
`lab/reports/locale-id-to-text.json`):

| Id | Name | Class | Category | Icon | Ability | Power | Cost |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 35 | Strong Arm Stew | 9 | 10 Food & Drink | 1016 | 3264 | 1 | 180 |
| 1195 | Wheeler on the Cob | 9 | 10 Food & Drink | 1241 | 232 | 1 | 10 |
| 2173 | Small Health Potion | 9 | 9 Potions | 730 | 267 | 42 | 15 |
| 3021 | Medium Health Potion | 9 | 9 Potions | 543 | 4697 | 3 | 25 |
| 11002 | Small Health Potion | 9 | 9 Potions | 730 | 4696 | 1 | 15 |
| 65820 | Wheeler Health Potion | 9 | 9 Potions | 749 | 4696 | 1 | 10 |
| 68989 | Health Potion | 9 | 9 Potions | 543 | 4340 | 1 | absent, serializes as 0 |

All seven are `Type 1`, single-use. `ItemCategories.txt` maps category 9 to name id 961
(`Potions`) and 10 to 962 (`Food & Drink`). Two distinct records share the displayed name
`Small Health Potion` (2173/267 and 11002/4696), so names alone cannot select an ability. The
recovered class 9 catalogue holds 337 potion/food definition rows; this piece carries the seven
proof-recorded candidates and does not claim catalogue coverage.

Captured 38/25 and 36/1 vectors used by the tests (all four SHA-256 hashes re-verified at run time):

| Vector | Bytes | SHA-256 |
| --- | --- | --- |
| `captured-medium-template.bin` (bar 2 slot 0, disabled Medium potion, quantity 59) | 63 | `0a4e41c4420868a044b6fb2caf3b1dfc7088013b4c4c67ba66205d16b79e1cee` |
| `empty-slot.bin` | 63 | `2d4e53d0ba2746a248ec6a89b070ff02b75157f96468737273ef6e09f2fb14ea` |
| `stew-inspection-slot.bin` (icon 1016, name 1232, quantity 1, never-cast sentinel) | 63 | `c7a8fa1df3c97d6862df8218c1ba59634022b7db43df88ac2e37bc7ed0726467` |
| `failure3079.bin` | 8 | `553173cd4ddf3e864a67625734ebe021ccf98e803abf4c2ade16efb108a5c457` |

## Effects and expiry evidence (cached wiki, secondary)

`lab/references/freerealms-wiki/wiki.sqlite`, verified by revision id and clean-text SHA-256:

| Page | Revision | Effects | Hash |
| --- | --- | --- | --- |
| Medium Health Potion | 10651 (2013-02-15) | 16%/s for 3 s + 2% instant | `e1e69e644a5a7127d1538ebb0dfffb10d00936a4d2a99073060aa82d40574e28` |
| Small Health Potion | 44062 (2021-01-20) | 11%/s for 3 s + 2% instant | `dd0b27548c279ae3bfcbb870d997872fce977f2f52513eae32dffbebc5c292a9` |
| Wheeler Health Potion | 10857 (2013-02-15) | 11%/s for 3 s + 2% instant | `7b4cc6964952e4b3ab9388b6a2980e00ebeefa1293b367521b637a494ef011ef` |

The original tooltip (string 422916) says "restores some of your health over 5 seconds". The
three-second wiki rows and the five-second tooltip disagree; no local original ability database
resolves it, so neither number is used as a rule. Strong Arm Stew has no recovered amount,
duration, size multiplier, stacking or cooldown value.

## Reconstructed policy (proposal, not original)

`ConsumableLedger` is an offline decision model over state that is already owned. Every duration is
caller-supplied through `UsePolicy`; the module bakes in no historical balance number. The rules,
in the order the module applies them:

1. Only the recovered item bar (id 2) is accepted; another bar is `WrongBar`.
2. The slot must have a recovered 42/6 assignment pointing at a known owned instance.
3. The assigned item must have a recovered catalogue row, otherwise `UncataloguedItem`.
4. A zero count is `Depleted`.
5. Any bit of the recovered client status mask blocks the use.
6. A request inside the caller-supplied recast window is `OnCooldown` with the remaining time.
7. A non-monotonic tick (equal to or older than the last accepted use) is
   `ReplayedOrStaleRequest`, which is the once-only guard for retransmission and replays.
8. An accepted use decrements exactly one item, starts one recast window and replaces that
   instance's effect window; `live_effects` and `ActiveConsumableEffect::expired` expire it on the
   exact `started + duration` boundary.

The community server (Sanctuary revision `4f028dcc274f628b65ef665fd59b184f5f3d3e3e`) demonstrates a similar but reconstructed
path: `ActionBarId = 2`, a 3000 ms minimum cooldown, a disabled slot packet followed by a scheduled
re-enable, and an effect record with expiry/buff icon/name fields. Those values are community
reconstruction and are not copied into the module as history; the module only records them here as
a cross-check that the request/response shape is understood.

## Evidence and checks

Test-first RED, then GREEN:

* RED (Rust): `.\scripts\cargo.ps1 test --manifest-path .\prototype\osfr-tools\Cargo.toml --test piece_046`
  failed with `error: couldn't find file tests\..\src\restoration\piece_046.rs`
  (`lab/reports/piece-046-20261009T0421Z/rust-red.txt`).
* GREEN (Rust): same command, 16 passed, 0 failed, 0 ignored, no warnings
  (`rust-green.txt`).
* RED (Python): `& ..\lab\tools\wiki-env\Scripts\python.exe -m unittest discover -s tests/pieces -p "test_piece_046.py" -v`
  failed to import `scripts/pieces/piece_046.py` (`python-red.txt`).
* GREEN (Python): 7 passed, 0 failed, including the real-source checker run
  (`python-green.txt`).
* Checker: `& ..\lab\tools\wiki-env\Scripts\python.exe scripts\pieces\piece_046.py --output <dir>`
  exits 0 with 12 passing checks over the pinned lab corpus (`checker-run.txt`,
  `piece-046-consumables.json`, `summary.md`).

The Rust vectors cover: exact 12/24/32/8-byte request encodings and their fail-closed decoders;
the three captured 38/25 slots parsed by the recovered offsets; the 36/1 failure vector; the
recast gate (age, zero refresh, never-cast sentinel); the input gate (status mask, mana, empty,
disabled); the seven catalogue rows; and the ledger behaviours (once-only consumption, cooldown
boundary, stale/replay refusal, refusals leaving quantity/effects/cooldowns untouched, single-use
depletion, effect expiry at the exact boundary, assignment validation). The Python checker binds
the module constants and catalogue to the item definitions, locale table, category rows, cached
wiki revisions, the four captured vectors, the recovered request framing and the decompiled
original enums/serializer order/Lua anchors, and fails closed on any drift.

## Boundaries and remaining gates

Not implemented or claimed: no item/currency/experience grant; no packet is sent; no server
registration of 42/6 or 36/10; no native slot refresh; no healing amount, duration, size or
stacking rule; no icon id to texture mapping; no consumable-to-effect-tag binding; no 38/25 target
metadata semantics; no Stew-specific target/cooldown recovery.

Integration gates for the lead: register `pub mod piece_046;` under `src/restoration/mod.rs` and
`src/lib.rs` before wiring it; note that `42/6` is currently a no-op and `36/10` ignores bar 2 in
the shared session code, which only the lead may change. A native acceptance run still needs a
fresh default session plus an unmounted disposable fixture (the borrowed toolbar template carries
`ForceDismount`), a real assignment capture, at least one real click per assigned slot, and an
explicit sourced or clearly labelled reconstruction decision before any effect number is applied.
Any actual economy change belongs to the lead-owned atomic layer.

Resume point: the module and checker are complete and green; the next bounded step is lead
integration of the wire codec, then a native assignment/activation capture to replace the
currently absent original Stew target/cooldown evidence.
