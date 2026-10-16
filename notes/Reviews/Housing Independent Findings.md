# Housing independent findings

Independent review and repair of the interrupted MMO006 housing candidate, 2026-10-08 19:50 Pacific.
Verdict: **the reconstructed state machine is sound after eight demonstrated defects were repaired
test-first. This is a code/oracle candidate only; there is no native housing acceptance and the
module is still unregistered.**

Reviewed paths: `prototype/osfr-tools/src/housing.rs` (SHA-256 `4F3A5EA6…` before, `3A971E9A…`
after) and `prototype/osfr-tools/tests/housing.rs` (`D67F1E09…` before, `2811982B…` after). No
shared file, manifest, gateway, content, save, launcher, release or other worker's path was edited.

Skills applied: RTK — commands ran normally and no result needed the `rtk proxy` fallback. Ponytail —
root-cause fixes at the shared layer and deletion of the two dead functions instead of documenting
them. Caveman — terse reporting. reverse-engineering — reading the recovered pseudocode and packet
model directly. game-automation, asset-pipeline and the remaining game skills were not applicable:
only the lead controls the native window, and no housing asset or wire path exists yet.

## Original evidence re-verified

Every ORIGINAL claim in [[Research/Housing Restoration]] was re-read at its source rather than trusted
from the note:

- base opcode **127** at `lab/decompiled/server/LibSOE/SOE/BasePackets.cs:78`; inner opcodes 1, 2, 3,
  5, 6, 28, 39, 40, 41 and 53 at `BaseHousingPackets.cs`;
- client constructors `FUN_00aa5340` → `FUN_00aa4990(1)` (pseudocode.c:1384376), `FUN_00aa53f0` →
  `FUN_00aa4990(2)` (1384411) and `FUN_00aa81a0` → `FUN_00aa4990(3)` (1386096);
- `FixtureInstance`, `PlayerHousingInstanceData` and `InstancePermission` field sets in the pinned
  community model, including the `// 3 - Can Edit` level;
- `Houses.json` SHA-256 `854B2D3F…` and row 1 (`Id` 1, `ZoneId` 1, `NameId` 6514, build area
  `[322.0, -100, 392.1]`–`[443.3, 60.0, 477.8]`), which the tests use verbatim;
- `ZoneManager.PlayerOwnsHouse` (`Sanctuary.Game/ZoneManager.cs:239,243`) and the client string id
  `HousingMaxFixtures^426393` (`CodeStringMappings.txt:1666`, read at pseudocode.c:1400456).

One label was overstated: the fixture cap `400` in the test fixture is a local value, not recovered
content, so the test was renamed to `placement_grants_ownership_and_reports_the_granted_limits` and
the helper comment now says which numbers are original.

## Demonstrated defects, fixed test-first

`lab/reports/housing-independent-20261009T0243Z/red-housing-tests.log` is the RED run before any
source change: 17 passed, 8 failed, one failure per code defect below (findings 7 and 8 are evidence
and API hygiene rather than test failures).

1. **High — a rejected snapshot load partially replaced live state.** `load_state` assigned the
   rejected document's content table and grants before returning `CorruptSnapshot`, so the live lots
   then validated against another document's limits. Demonstrated by a rejected document whose
   `max_fixtures` 0 blocked an otherwise legal placement on the live lot
   (`a_rejected_snapshot_load_leaves_the_live_state_intact`).
2. **High — duplicate lot guids in a snapshot silently dropped a lot.** `rebuild` inserted into a map,
   so two rows sharing one guid collapsed to the last row and the first row's fixtures vanished.
   Now rejected (`duplicate_lot_guids_in_a_snapshot_are_rejected`).
3. **Medium — a replaced lot cap or build area diverged from the content table.** After
   `replace_saved_fixtures`, placement used stale property limits: an over-cap placement was accepted
   and then could not be saved, and a replacement build area was ignored for placement. The hook now
   validates the replacement, requires the lot's identity to be unchanged and realigns the content
   table (`a_lot_limit_change_is_enforced_by_placement_and_still_saves`,
   `a_lot_build_area_change_is_enforced_by_placement`).
4. **Medium — the save path trusted a client house guid.** A `SaveFixtureCommand` whose placement
   named another house persisted a fixture that `validate_snapshot` later rejected, so an accepted
   save made the lot unsaveable. The server now stamps the resolved lot's guid, and empty instance or
   definition ids are refused with `InvalidPlacement` like the placement path
   (`save_fixture_placement_stamps_the_resolved_lot_identity`,
   `save_fixture_placement_rejects_an_empty_instance`).
5. **Medium — a save that rewrote an existing fixture skipped the build-area bound** that both
   `move_fixture` and first-save enforce, letting a client persist an out-of-area fixture. It now
   shares the same check and leaves the stored transform untouched when denied
   (`saving_an_existing_fixture_keeps_the_build_area_bound`).
6. **Low — a rejected `FixtureItemLedger::seed` partially consumed ids**, so the same ids could never
   be seeded afterwards. The whole request is validated before mutation
   (`a_rejected_seed_leaves_the_ledger_unchanged`).
7. **Evidence hygiene.** The candidate note claimed RED-first text and a per-target sweep were
   recorded in `lab/reports/housing-restoration-20261008T191409Z/`, but that directory held only one
   full-suite log. The claim is corrected and the missing receipts are recorded under this review's
   own directory.
8. **Dead, misleading API removed.** `Housing::grant_property_or_default` never granted anything (it
   only re-derived a lot guid) and the private `mutable_lot` was unused; both were deleted.

Observations that were left alone: `HousingError::NotPermitted` and `LandmarkLimitReached` are never
constructed because permissions and landmark limits are carried but not enforced, which the research
note already states as a gap.

## Verification after repair

- Housing target: **26 passed, 0 failed** (`green-housing-tests.log`), including the restart
  conservation case, which pins the recipe: reseed captured stock, re-stage the lot's persisted
  fixtures into the ledger, then move or pick up. A pickup with no staged instance is still refused,
  so a restart cannot mint furniture. A guest granted level-3 permission may edit while a stranger
  may not, and only the owner may grant or lock.
- Scoped `rustfmt --check` on the two owned files: exit 0. No crate-wide reformat was run.
- Per-target sweep (`per-target-sweep.log`): every target that compiled cleanly passed, including
  `housing` at 26. Unowned in-flight targets were mid-edit during the sweep (`derby` failed four of
  its own tests; `piece_009` and `piece_017` could not include a module their workers had moved).
  None of that flowed from this change: `housing.rs` is included only by `tests/housing.rs`.

## Boundaries and next gates

No native client has seen this module. There is still no housing packet writer, no decoded body for
`HousingPacketFixturePlacementDenied`, and no collision, snap, drag, surface, landmark, upkeep,
visitor, customization or interactive-furniture behavior. Placement, pick-up and save remain
RECONSTRUCTED rules over ORIGINAL packet, property and field facts. Native acceptance, `lib.rs`
registration, inventory-transaction integration, property acquisition and save migration stay with
the lead and items 021–030.

Evidence: `lab/reports/housing-independent-20261009T0243Z/` — `red-housing-tests.log`
SHA-256 `157E5D0F…`, `green-housing-tests.log` `985AB295…`, `per-target-sweep.log` `BDE8786F…`,
`full-suite.log` `7BC12F87…`, `source-verification.log` `6868A32B…`, `pre-change-hashes.txt`
`15ECDB36…`, `final-hashes.txt` `373EEA20…`.

Cross-links: [[Research/Housing Restoration]], [[Design/MMO Worker Contract]], [[MMO Work Queue]],
[[Gameplay Coverage]].
