# 028 Housing save migration

Status: versioned restore and invalid/old-data handling implemented and locally verified,
2026-10-09 03:45 UTC. Evidence label: **RECONSTRUCTED version boundary over ORIGINAL packet and
property facts; no original save format recovered**. No native client has accepted anything this
module writes, and the item 006 housing module is still awaiting lead registration.

Owned paths:

| Path | Role |
| --- | --- |
| `prototype/osfr-tools/src/restoration/piece_028.rs` | version dispatch, legacy upgrade, staged restore |
| `prototype/osfr-tools/tests/piece_028.rs` | 14 owned vectors, `#[path]` include |
| `scripts/pieces/piece_028.py` | independent migration/conservation checker |
| `tests/pieces/test_piece_028.py` | 11 stdlib checker tests |
| `lab/reports/piece-028-20261009-0345Z/` | RED/GREEN logs, exported artifacts, checker summary |

No shared `lib.rs`, gameplay, gateway, Cargo, launcher, content, save or release path was touched.

## Why this item exists

Item 006 (`notes/Research/Housing Restoration.md`) writes one JSON document, `schema_version` **1**,
whose `load_state` refuses every other version with a single `CorruptSnapshot` error and never
rewrites the file it read. Its own next-gates list names this item: "versioned restore of older
snapshots once real saves exist. The current schema rejects an unknown `schema_version` and requires
an exact `count`." Two consequences follow, and both are what this module closes:

1. a document that predates the fields the item 006 review added (`count`, `build_areas`) cannot be
   restored at all, so an operator upgrading a running server has no path that keeps a lot's
   furniture;
2. a document a *newer* build wrote is refused with the same error as a corrupted one, and nothing in
   the refusal guarantees the operator does not overwrite it with a fresh empty state.

The RED evidence is the first revision of `tests/piece_028.rs` running against a strict
current-version-only module: every legacy vector failed with
`UnsupportedVersion { found: 0 }` while the current-version vectors passed. The log is
`lab/reports/piece-028-20261009-0345Z/red-tests.log` (`test result: FAILED. 5 passed; 9 failed`).

## Original facts used

Nothing below is invented; every claim is read from the supplied client, the decompiled community
server, the pinned community resources or the repo's own earlier work.

- **The local snapshot mirrors the recovered instance payload.** `PlayerHousingInstanceData` carries
  the house guid, owner guid, owner name, name id/name, `MaxFixtureCount`, `MaxLandmarkCount`, the
  fixture map, the permission map, `CurFixtureCount`, `BuildAreas`, icon, furniture score and
  members-only flag, and `FixtureInstance` carries the fixture guid, house guid, fixture definition
  id, position, rotation, tilt, npc guid, tint id, customization, xml and scale
  (`lab/references/FreeRealms-Legacy/src/Sanctuary.Packet.Common/Housing/PlayerHousingInstanceData.cs`).
  The document this module migrates is the item 006 mirror of that field set.
- **`count` is derived data.** `PlayerHousingInstanceData.CurFixtureCount` counts the same fixture map
  the document stores, so the fixture list inside the document is the authoritative source for it.
  That is the only field this migration recomputes.
- **Build areas are original content.** `Resources/Houses.json` (sha256
  `854b2d3f009b916a928f66d57a42e2bf685dcc853e898d149580eefd26196d3c`, 31 rows, verified again this
  run) carries each property's `Id`, `ZoneId`, `NameId`, icon, spawn pose and `BuildAreas` min/max
  pairs. Rows 1, 2 and 3 are used verbatim in the vectors; the module never invents a rectangle, it
  refuses a lot whose property id it cannot resolve.
- **The save action exists on the wire; its body is only partly recovered.** The decompiled server
  enum names `ClientHousingPacketSaveFixture = 5` inside the family
  (`lab/decompiled/server/LibSOE/SOE/BaseHousingPackets.cs`), and item 025 recovered the shipped
  client's writer `FUN_00abf620` (`7f 00 05 00`, `u64`, 4×`u32`, 4×`u32`, `u32`, undecoded block,
  `u32`) from `lab/decompiled/client/FreeRealms/pseudocode.c`. The undecoded tail means no packet can
  be translated into or out of this document yet; that stays item 025's gap.
- **No original server save exists to migrate from.** The pinned community server has no housing
  persistence anywhere: `Sanctuary.Database/DatabaseContext.cs` declares `Users`, `Items`, `Titles`,
  `Mounts`, `Guilds`, `Friends`, `Ignores`, `Profiles`, `Characters`, `GuildMembers` and
  `CharacterQuests` and no housing set; none of the eight MySql migrations
  (`20250823191021_Initial` … `20260915000000_AddCharacterQuests`) touches a house or fixture table;
  a case-insensitive search of the Database projects for `House`/`Housing`/`Fixture` returns nothing;
  `Sanctuary.Gateway/Handlers/BaseHousingPacketHandler.cs` answers every unimplemented housing opcode
  with the literal comment `housing not implemented`; and `ClientHousingPacketEnterRequestHandler` is
  a stub whose real logic is commented out. There is therefore **no real save format in the
  evidence**, only reconstructed local documents.
- **The repo already has this migration convention.** `src/game_store.rs` loads version 1
  position-only saves and version 2 snapshots under one loader and upgrades in memory "without
  resetting progress". This module applies the same convention to the housing document.

## What is reconstructed

Everything below is a local rule with an explicit ceiling, not a recovered behaviour.

- **Legacy version `0`.** A legacy document either omits `schema_version` or declares `0`. Its
  per-lot shape is the current `housing::LotSnapshot` minus the two fields the item 006 review added
  (`count`, `build_areas`); both are optional when reading a legacy row, because a legacy writer may
  have carried either. No historical file of this shape is claimed to exist — it is the mechanical
  repair rule, and `tests/pieces/test_piece_028.py` derives the same shape independently.
- **A migration may only add derived data.** `count` is recomputed as the fixture-list length (an
  explicit legacy count that disagrees is *corrected*, and the correction is counted in the report);
  `build_areas` are filled from the recovered property rows only when the legacy lot has none, and
  only when that row's `zone_id`/`name_id` agree with the lot's own values. Identities, ownership,
  property ids, fixture guids, instance ids, transforms, permissions and the lock flag are copied
  verbatim and are asserted unchanged.
- **The canonical output is item 006's own type.** The migrated document is serialized from
  `housing::LotSnapshot`/`housing::SavedFixture`, so it cannot drift from what `Housing::save_state`
  writes; the compile-time `assert!(CURRENT_SCHEMA_VERSION == 1)` fails the build if item 006 bumps
  its version without a migration arm here.
- **Version dispatch is on the raw JSON.** A missing version is legacy; a non-negative integer is
  compared against the current version; anything else is `VersionNotInteger`; anything newer is
  `UnsupportedVersion { found }` and the source must not be overwritten. A *current-version* document
  that is missing `count` or `build_areas` is `InvalidDocument` and is deliberately **not** repaired —
  repair is only for a document that declared itself legacy.
- **Restore stages through the real loader.** `restore` reads the source (bounded to
  `MAX_SNAPSHOT_BYTES`), migrates in memory, writes the migrated bytes to a same-directory staging
  file, and hands that path to `Housing::load_state`. The loader's staged design means a refused
  document cannot partially adopt; the staging file is removed on every path. The source file is
  never rewritten, and this module never touches the item 026 owned-furniture pool, so no unit can
  move between placed and available during a migration.
- **Deliberate boundaries.** `restore` needs a writable directory because of the staging file (the
  same directory the server already saves into). There is no down-conversion, no wire migration and
  no "empty document wipes live state" guard: item 006 writes snapshots with atomic replacement, so a
  valid-but-empty document can only come from a server that owned no lots, and refusing it would break
  first start. That refusal is deliberately not implemented rather than half-implemented.

## Verification

Commands run from `prototype/osfr-tools` through `scripts/cargo.ps1` (private toolchain under
`../lab/tools`), and from the project root with `../lab/tools/wiki-env/Scripts/python.exe`.

| Command | Result | Log |
| --- | --- | --- |
| `cargo.ps1 test --test piece_028` (strict first revision) | 5 passed, 9 failed — every legacy vector refused with `UnsupportedVersion { found: 0 }` | `red-tests.log` |
| `cargo.ps1 test --test piece_028` | 14 passed, 0 failed, 0 ignored, no warnings | `green-tests.log` |
| `cargo.ps1 test --test housing --test piece_022 --test piece_025 --test piece_026 --test piece_027` | 26 + 12 + 8 + 12 + 12 passed | `neighbour-suite.log` |
| `cargo.ps1 test --lib` | 24 passed, 0 failed | `lib-suite.log` |
| `python scripts/pieces/piece_028.py --legacy … --migrated … --report … --houses Houses.json --stock … --future …` | exit 0, `ok: true`, 2 lots / 3 fixtures, `derived_counts` 2, `filled_build_areas` 2, conservation 4/1 owned and 2/1 placed for definitions 17310/17311 | `independent-checker.log`, `checker-summary.json` |
| `python -m unittest discover -s tests/pieces -p "test_piece_028.py" -v` | 11 passed | `python-tests.log` |
| `python -m unittest discover -s tests/pieces -p "test_piece_02*.py"` | 100 passed (the neighbouring checker tests are unaffected; the `libpcap` warning comes from a neighbour's capture helper) | `python-neighbour-suite.log` |
| `rustfmt --edition 2024 --check src\restoration\piece_028.rs tests\piece_028.rs` | clean for both owned files | `fmt-check.log` |
| `cargo.ps1 fmt --check` (whole crate) | reports formatting diffs owned by other in-flight workers (`tests/piece_036.rs`, `src/restoration/piece_009.rs`, …); no `piece_028` file is listed | `fmt-check-crate-wide.log` |
| `cargo.ps1 test` (whole crate) | **cannot complete**: `tests/piece_035.rs` and `tests/piece_038.rs` include `src/restoration/piece_035.rs`/`piece_038.rs`, which do not exist yet. Not this item's failure | `full-suite.log` |

Two findings from the red-to-green cycle changed the module or the vectors rather than the tests'
intent:

1. a legacy document that declares `schema_version: 0` explicitly was first rejected by the legacy
   reader (`deny_unknown_fields` had no version field), so the legacy shape now accepts the field and
   re-checks it; versionless and explicit-zero documents now migrate to byte-identical output;
2. `Housing::load_state` collapses every `validate_snapshot` failure into `CorruptSnapshot`, so a
   duplicate-property document is observably `Refused(CorruptSnapshot)` — the vector asserts the
   refusal and the untouched live state, not a specific inner variant.

The Python checker is a second implementation in another language. It re-derives the migrated
document from the exported legacy file and the ORIGINAL `Houses.json`, compares it to the Rust output,
replays the per-lot preservation rule, checks the item 026 pair invariant over the exported pool, and
verifies the report's numbers and both source hashes (before/after) — including the future-version
refusal. It found no disagreement with the Rust module.

## Deliberate gaps and remaining gates

1. **No original save format.** The community server stores no housing row and the original server is
   not preserved, so there is nothing historically authentic to migrate *from*. The legacy version 0
   here is a local reconstruction; the exact missing evidence is a real pre-existing housing save
   document (client-side or server-side) with a different shape than item 006's writer.
2. **Wire (item 025).** `ClientHousingPacketSaveFixture`'s nested block is undecoded, so a packet
   cannot yet be turned into a snapshot or back. Nothing here answers the client.
3. **Lead integration.** Register `housing` and this module in `src/lib.rs`, then replace the
   `#[path]` include here and in `tests/piece_028.rs` with `use crate::housing;`. On startup the
   server should call `restore` before serving, and may re-save the canonical form with
   `Housing::save_state` when the report says `migrated`.
4. **Owned-furniture pool (item 026).** This module deliberately leaves the pool alone; the pair
   invariant is checked read-only by the Python checker. Item 026's transaction remains the only
   writer of that file.
5. **Native acceptance (item 030).** Only entering a lot from the native client, placing, moving,
   picking up and re-entering after a restart proves the client accepts the reconstructed rules.
   This item's evidence is server-side only.

Cross-links: [[Housing Restoration]], [[026 Housing inventory transactions]],
[[025 Furniture placement client wire]], [[../../MMO Work Queue]], [[../../Design/MMO Worker Contract]].
