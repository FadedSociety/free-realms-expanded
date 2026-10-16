# 057 Loot and enemy rewards

Candidate artifact for queue item 057. It carries recovered loot evidence, the pinned community
weighted-drop schema, five wiki-documented enemy drop lists with their explicit gaps, and a
reconstructed once-only kill/loot grant ledger. Nothing is registered in `lib.rs`, no gateway or
content file changed and no native client run happened: the shared game window is lead-controlled,
so native acceptance is **pending**.

## What the slice ships

| Path | Role |
| --- | --- |
| `prototype/osfr-tools/src/restoration/piece_057.rs` | Recovered constants, documented lists, weighted `DropTable`, `LootLedger` |
| `prototype/osfr-tools/tests/piece_057.rs` | `#[path]`-included focused tests (11) |
| `scripts/pieces/piece_057.py` | Read-only corpus oracle (107 checks) |
| `tests/pieces/test_piece_057.py` | Hermetic + real-corpus tests (19) |
| `../lab/reports/piece-057-20261009T054124Z/` | Green oracle run over the formatted module (`piece-057-check.json`, `piece-057-provenance.json`) |
| `../lab/reports/piece-057-20261009T053524Z/` | Retained first run: 102/107, the checker parser missed the `COINS_NAME_ID` constant |
| `../lab/reports/piece-057-20261009T053629Z/` | Intermediate green run before `rustfmt` reformatted the carrier |

## Original facts recovered

Loot event opcode. The preserved original client pseudocode
(`lab/decompiled/client/FreeRealms/pseudocode.c`, sha256 `f84a5867…`) contains the player-update
constructor table: `FUN_0089fbb0(0x1d);` immediately before `PlayerUpdatePacketLootEvent::vftable`.
`0x1d` is sub-opcode **29**, which the unchanged community
`lab/decompiled/server/LibSOE/SOE/BasePlayerUpdatePackets.cs:33` enum names
`PlayerUpdatePacketLootEvent = 29` independently. The same table is self-consistent for the whole
family (`12 NpcRelevance`, `35 HitPointModification`, `41 AddEffectTagCompositeEffect`, …), which
is why it is treated as recovered evidence rather than a naming guess. Constructor shape only:
class offset `0x30` is an `IString` and offsets `0x40`/`0x44` take two integer parameters. No
serializer or field semantics were recovered, so this slice writes **no** loot packet bytes.

Loot props. `lab/references/Sanctuary/src/Resources/Models.txt` (sha256 `0bad5a15…`, a
client-derived model table) rows before the model id: `720^loot_chest_01.adr`,
`807^loot_bag_big_01.adr`, `808^loot_bag_small_01.adr`, `841^loot_coins_01.adr`. The original
client's own `Assets_manifest.txt` (sha256 `935bc0d9…`) lists `loot_bag_big_01.adr`,
`loot_bag_small_01.adr`, `loot_chest_01.adr.z` and `loot_coins_01.adr.z`, so the assets are real
client content, not a community invention. Extracted originals of three of them sit in
`lab/extracted/community-pack-check/Assets_008/` (`loot_bag_small_01.adr`,
`loot_bag_big_01.dma`, `loot_coins_01.cdt`); no extracted `loot_chest_01` file is in this lab
slice, only the manifest row.

Loot text. Locale rows (from `lab/reports/locale-id-to-text.json`, sha256 `f1f22a30…`) id 2
`You receive #count([*item*])`, id 3 `You receive #count([*experience*]) and #count([*coins*])`
and id 4923 `Coins`.

## Community evidence, pinned (not original)

* `lab/references/Sanctuary/src/Sanctuary.Core/Collections/WeightedDropTable.cs`
  (sha256 `a25fdd01…`): nonempty table, positive weights, `int` total cap, roll in `0..total`,
  cumulative walk. `DropTable` mirrors exactly these rules and no others.
* `lab/references/Sanctuary/src/Sanctuary.Game/RewardManager.cs` (sha256 `bc4398d2…`):
  lowercase key lookup, roll, item/currency grant switch, refusal of an unknown item definition.
* `lab/references/Sanctuary/src/Resources/Rewards.json` (sha256 `0b0296ac…`): the only weighted
  table in the pinned corpus - `dev-test-reward`, item 205 ×1 weight 1 plus 50 coins weight 1.
  It is a hand-authored community dev fixture and is used here as a schema fixture, **not** as
  original drop data.
* Five preserved wiki pages document an enemy drop list with the sentence "Once defeated, there is
  a chance to receive the following items". The lists are player-observed names only - no
  probabilities, quantities or coin amounts - and each page carries its revision hash in the
  oracle report. The `Sandspear` page says "Drops from Chest", which is the only reason the
  reconstructed prop plan uses the chest model for a boss.

| Wiki page | Page id | Revision | Entries | Names that resolve to a locale row → captured item row |
| --- | --- | --- | --- | --- |
| Floren Defender | 1035 | 48775 | 18 | 15 of 17 item names resolve to locale ids (442929, 442930, 442931, 442932, 442933, 442934, 442942, 442947, 442948, 442954, 442956, 442062, 442064, 442065, 442067); none join an item row |
| Sandscale Magus | 2377 | 45373 | 2 | Manguana Tribal Necklace → 8459 → item 18172 |
| Sandscale Scavenger | 2378 | 45371 | 5 | Manguana Tribal Necklace → 8459 → item 18172; Cloth Wraps → 34006 (no item row) |
| Sandspear | 2383 | 50196 | 5 | Manguana Tribal Necklace → 8459 → item 18172; Spiked Gloves → 34012 (no item row) |
| Skullz Roughrider | 2520 | 45434 | 5 | Skullz Biker Badge → 8462, Buckle-strapped Shirt → 33988 (no item rows); Coyote Juice Bottle → 33952 → item 38822; Dusty Bones Potion → 33962 → item 38827 |

## Gaps, stated explicitly

* **No original enemy drop table was recovered.** The decompiled Gateway/LibSOE carries the
  sub-opcode enum but no loot handler, the supplied `OSFRServer` has none either, and the captured
  `PacketSendSelfToClient.json` reward bundles are collection/quest rewards, not kill loot. Every
  enemy weight, quantity and coin amount is therefore **unknown**, and
  `weighted_from_documented()` refuses to build a table from a documented list.
* **No loot packet layout.** Sub-opcode 29 and the constructor shape are recovered; the field
  semantics and serializer are not, so no loot event bytes are produced.
* **No XP-per-kill value.** Locale 3 mentions experience in the loot text; no amount was recovered.
* Of the 35 documented entries (5 are "Coins"), 30 are item names: 5 of those 30 entries join a
  captured item definition (3 distinct names - Manguana Tribal Necklace → 18172, Coyote Juice
  Bottle → 38822, Dusty Bones Potion → 38827), 25 have no captured item row, and 6 of the 25 have
  no locale row at all (two each on Floren Defender, Sandscale Scavenger and Sandspear). The
  oracle asserts those absences so a future corpus addition shows up as a changed check.

## Reconstruction boundary

RECONSTRUCTED, labelled in code: the ledger policy (only a registered defeat can be looted, only
the credited killer is eligible, one grant per kill), the `KillId` shape (npc definition id, actor
guid, spawn generation) and the prop plan (normal enemy → small bag plus coins, boss → chest).
The original group/party sharing rule belongs to work item 063 and is deliberately absent; the
caller supplies the roll, so no distribution is invented here. The module grants nothing by
itself - it returns `LootOutcome` decisions and never touches inventory, currency or saves.

## Checks run

```
# RED, before the module existed
prototype/osfr-tools> powershell -File ..\..\scripts\cargo.ps1 test --test piece_057
error: couldn't find file `tests\..\src\restoration\piece_057.rs`

# GREEN
prototype/osfr-tools> powershell -File ..\..\scripts\cargo.ps1 test --test piece_057
test result: ok. 11 passed; 0 failed

project> python scripts\pieces\piece_057.py --out ..\lab\reports\piece-057-<UTC>
107/107 checks pass, 0 problems

project> python -m unittest discover -s tests\pieces -p "test_piece_057.py" -v
Ran 19 tests ... OK

prototype/osfr-tools> rustfmt --check --edition 2021 src/restoration/piece_057.rs tests/piece_057.rs
exit 0
```

The oracle is read-only over the corpus and hashes ten inputs into
`piece-057-provenance.json`; the hermetic tests tamper one field at a time (opcode, a documented
list, a wiki revision, a bullet list, a name gap, an item join, a model row, a manifest entry, a
pseudocode anchor, a pinned semantics anchor, the fixture weights) and require the matching check
to fail.

## Remaining integration and native gates

1. Lead registers `restoration::piece_057` in the shared module list and drops the `#[path]`
   include in the test.
2. Encounter integration is the lead's call: the natural hook is the defeat branch that already
   emits `35/3` removal - register the kill, then spawn the recovered prop model and apply the
   returned grant. No packet for that spawn is claimed by this slice.
3. Native acceptance (game-automation, lead-controlled): visible loot prop, pickup, loot toast and
   the real sub-opcode 29 event, plus a duplicate-kill replay proving one grant. Not exercised.
4. Any future recovery of original enemy weights, amounts or the loot serializer replaces the
   documented lists; until then the wiki lists stay observations and stay unweighted.

## Skills applied

RTK (normal commands first), ponytail (reuse the pinned schema; smallest engine that satisfies the
acceptance; no speculative packets), caveman (concise reporting while this note stays complete),
game-research-websearch (pinned community repo and the read-only wiki corpus), reverse-engineering
(preserved Ghidra pseudocode constructor table) and game-recon (engine/asset provenance: client
manifest, model table, packs). `asset-pipeline` and `game-automation` were not applicable: this
slice produces no new asset and owns no native run.
