# 026 Housing inventory transactions

Status: scoped transaction implemented and locally verified, 2026-10-09 02:52 UTC. Evidence label:
**RECONSTRUCTED server transaction over ORIGINAL packet and property facts**. This is not a native
housing acceptance claim; no client has accepted anything this module writes.

Owned paths:

| Path | Role |
| --- | --- |
| `prototype/osfr-tools/src/restoration/piece_026.rs` | transaction boundary, owned-furniture pool |
| `prototype/osfr-tools/tests/piece_026.rs` | 12 owned vectors, `#[path]` include |
| `scripts/pieces/piece_026.py` | independent conservation checker |
| `tests/pieces/test_piece_026.py` | 10 stdlib checker tests |
| `lab/reports/piece-026-20261009T025209Z/` | RED/GREEN logs, checker log, exported pair |

No shared `lib.rs`, gameplay, gateway, Cargo, launcher, content, save or release path was touched.

## Why this item exists

Item 006 (`notes/Research/Housing Restoration.md`) implements placement and pick-up as a bounded
state machine with an in-memory `FixtureItemLedger`, and its own note lists inventory integration as
the next gate: "map `FixtureItemLedger` proposals onto the lead-owned inventory transaction so a
placement and its stack change commit or roll back together".

The 006 module commits a placement in memory and only then lets the caller persist. A failing
snapshot write therefore leaves the placed fixture and the consumed unit live in memory while the
durable state says nothing happened. That is the defect this item closes, and it is the RED
evidence: the first revision of `tests/piece_026.rs` used the 006 module directly and failed with

```
assertion `left == right` failed: a fixture whose snapshot could not be written must not stay placed
  left: 1
 right: 0
```

Retained at `lab/reports/piece-026-20261009T025209Z/red-housing-only.log`.

## Original facts used

Everything below is read from the supplied client, the decompiled community server model, the pinned
community resources or the read-only wiki corpus. Nothing here is invented.

- Housing is packet family **127**. `ClientHousingPacketPlaceFixtureRequest` = 1,
  `ClientHousingPacketPlaceFixture` = 2, `ClientHousingPacketPickupFixture` = 3,
  `ClientHousingPacketSaveFixture` = 5, `HousingPacketFixtureRemove` = 41 and
  `HousingPacketFixtureItemList` = 43
  (`lab/decompiled/server/LibSOE/SOE/BaseHousingPackets.cs`).
- The supplied client constructs the fixture item list (`HousingPacketFixtureItemList::vftable`,
  `lab/decompiled/client/FreeRealms/pseudocode.c` line 1396199) and reads `HousingMaxFixtures` from
  the property definition (line 1400456, next to the `Fixture placement denied.` string).
- `HousingPacketFixtureItemList` carries `FixtureInstanceInfo` (fixture guid plus `ItemDefinitionId`
  and coupling display) and `FixtureDefinition` (fixture id, `ItemDefinitionId`, model id, category,
  `LuaCall`, actions, composite effect). A fixture definition is tied to an **item definition**, so
  the client sees owned furniture items next to the lot's placed fixtures
  (`lab/references/FreeRealms-Legacy/src/Sanctuary.Packet/BaseHousingPacket/HousingPacketFixtureItemList.cs`,
  `.../Sanctuary.Packet.Common/Housing/FixtureInstanceInfo.cs`, `FixtureDefinition.cs`).
- `PlayerHousingInstanceData` carries `MaxFixtureCount`, `CurFixtureCount`, `CurLandmarkCount`, the
  fixture map, permissions, build areas and the furniture score; `FixtureInstance` carries the
  fixture guid, house guid, fixture definition id, position, rotation, tilt, tint id and scale
  (`.../Housing/PlayerHousingInstanceData.cs`, `FixtureInstance.cs`).
- Property row 1 of `Resources/Houses.json` (sha256
  `854b2d3f009b916a928f66d57a42e2bf685dcc853e898d149580eefd26196d3c`) gives property 1, zone 1, name
  6514 and a build area of x 322.0..=443.3, y -100.0..=60.0, z 392.1..=477.8. The Rust vectors use
  that row verbatim, as item 006 does.
- The read-only wiki corpus records lots as owned housing areas the player builds on and furniture
  as indoor and outdoor items (`Lots`, revision 49783, cited in
  `notes/Research/Housing Restoration.md`; the `Furniture` category pages, for example revision
  43345, describe housing furniture bought or granted as items).

Not recovered, and therefore not claimed: whether the original server consumed an item instance at
placement time, which item instance identity the client sends with
`ClientHousingPacketPlaceFixtureRequest`, the payload of `HousingPacketFixturePlacementDenied`, and
any original repaint or pick-up tint rule. The community gateway implements only
`ClientHousingPacketSetEditMode` and `ClientHousingPacketEnterRequest`, so there is no community
reference implementation for place, pick-up or save to cross-check.

## What is reconstructed

All of the following is local reconstruction with an explicit ceiling, not a recovered rule.

- **Durable owned pool.** A second document next to the item 006 housing snapshot
  (`furniture-stock.json`, schema version 1) holds one row per owned furniture unit:
  `{instance_id, definition_id, tint_id}`, sorted by `(definition_id, instance_id)` and unique on
  that pair. The pool bound is 8192 rows and 256 KiB.
- **Derived ledger.** `FixtureItemLedger` is not stored. It is rebuilt from the pair on every
  operation: every pool row seeds its bucket, and every placed fixture is staged, so the ledger item
  006 stages against can never show more available units than the durable pair records.
- **Placement and pick-up write one file.** A placement or pick-up changes only the placed/unplaced
  split, so only the housing snapshot is written. The pool document is byte-identical before and
  after a placement, a pick-up and a move; the vectors assert that.
- **Tints.** A unit's stored tint is authoritative only while the unit is unplaced. While placed,
  the fixture's tint is authoritative, because item 006's `rekey_tint`/`restore` pair returns a
  picked-up unit to the tint it was placed with. A pick-up that changed the tint writes the pool
  first and the lot second.
- **Commit ordering and rollback.** A failed housing write restores the lot in memory and reports
  `PersistHousing`; a failed pool write restores the pool in memory and reports `PersistStock`. Both
  files are written by same-directory temporary-file replacement with a bounded Windows
  sharing-violation retry, so a failed write never replaces the destination. When a pick-up already
  wrote the pool and the lot write then fails, the previous pool bytes are written back; if that
  compensating write also fails the error is `RollbackFailed`, and the reported residue is harmless
  because a placed unit's tint follows its fixture.
- **Refusals are not commits.** `Rejected` means the state machine refused the request: nothing
  changed in memory and both files stay byte-identical. The vectors assert that for a stranger, an
  out-of-area transform and an unowned label.
- **Conservation.** For every definition, `owned == placed + available` and `placed <= owned`; every
  placed fixture's `(definition, instance_id)` must be a pool row; per lot `CurFixtureCount` must
  equal the stored fixture map; a fixture's house guid must equal its lot; a property id may be
  owned by one lot only. A pair that breaks any of these is refused at load.
- **Consuming a placed unit is refused.** `consume_instance` reports `InstanceInUse` for a unit that
  is currently placed, which is what keeps placement and removal conservative rather than
  approximate.
- **Ambiguous placement tints are refused.** Item 006's pool is a count, so one unit placed in two
  lots at two different tints cannot be represented. That state is reported as
  `AmbiguousPlacedTint` and the move is rolled back instead of being silently folded.

## Verification

Commands run from `prototype/osfr-tools` with `scripts/cargo.ps1` (private toolchain under
`../lab/tools`), and from the project root with `../lab/tools/wiki-env/Scripts/python.exe`.

| Command | Result | Log |
| --- | --- | --- |
| `cargo.ps1 test --test piece_026` (first revision, 006 module only) | 0 passed, 1 failed - the RED assertion above | `red-housing-only.log` |
| `cargo.ps1 test --test piece_026` | 12 passed, 0 failed, 0 ignored, no warnings | `green-final.log` |
| `cargo.ps1 test --lib` | 24 passed, 0 failed | `lib-suite.log` |
| `cargo.ps1 test --test housing --test inventory --test character_store` | 26 + 12 (+2 ignored) + 8 passed | `neighbour-suite.log` |
| `python scripts/pieces/piece_026.py --housing lab/reports/piece-026-20261009T025209Z/pair/housing.snapshot.json --stock .../pair/furniture-stock.json --consensus .../pair/conservation.json` | exit 0, `conserved: true`, owned 6 / placed 2 / available 4, no errors | `independent-checker.log` |
| `python -m unittest discover -s tests/pieces -p "test_piece_026.py" -v` | 10 passed | `python-tests.log` |

The independent checker is a second implementation of the invariant in another language. It reads
the two files the Rust suite exported (`lab/reports/piece-026-20261009T025209Z/pair/`) and derives
the same balances the Rust module reports in `conservation.json`, including the repainted chair
(fixture tint 7) and the table at tint 0.

The crate-wide `cargo test` did not complete: the concurrently written `tests/piece_008.rs` (chess,
another worker's exclusive path) does not compile against its module, so cargo stops before other
targets. That failure is unrelated to this item; the two files this item owns compile and pass on
their own, and the neighbouring suites above pass. Full log: `full-suite.log`.

## Deliberate gaps and remaining gates

1. **Lead integration.** Register `pub mod housing;` and the restoration module in
   `src/lib.rs`, then replace the `#[path] = "../housing.rs"` include in
   `src/restoration/piece_026.rs` with `use crate::housing;`. The re-export block above it exists
   only so the owned test target compiles before registration.
2. **Real inventory (item 081).** The pool here counts owned furniture units. The lead's
   `inventory::Inventory` still holds real stacks and currencies; `grant_instance` and
   `consume_instance` are the seams that must be driven by that store, together with the housing
   write, before placement changes a real backpack.
3. **Wire (item 025).** `ClientHousingPacketPlaceFixtureRequest`, `ClientHousingPacketSaveFixture`,
   `ClientHousingPacketPickupFixture` and `HousingPacketFixturePlacementDenied` bodies are still
   undecoded, so nothing here answers the client. Until item 025 lands, this module is exercised by
   its own vectors only.
4. **Ownership (item 021).** `grant_property` is an in-memory grant; purchase, currency accounting
   and once-only ownership are item 021's scope and are not modelled here.
5. **Native acceptance (items 030, 027).** Enter a lot from the native client, place, move, pick up
   and re-enter after a restart. Only that proves the client accepts the reconstructed rules; this
   item's evidence is server-side only. Collision, snap, landmark scoring and upkeep remain item
   006's declared gaps and are not addressed here.
6. **Two-file ceiling.** A pick-up that changes a tint writes the pool and then the lot. A crash
   strictly between the two writes leaves the pool tint advanced while the lot still shows the
   fixture; the pair still loads and still conserves, but a single-file journal would be needed for
   full crash atomicity. Marked in the module as a `ponytail:` ceiling rather than hidden.

Cross-links: [[Housing Restoration]], [[../../MMO Work Queue]], [[../../Design/MMO Worker Contract]].
