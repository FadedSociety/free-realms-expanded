---
updated: 2026-10-17
---

# Housing Path

Status: research only. No Rust, content, launcher, save, or native-client change
was made for this note.

## Short answer

The 1,273 missing files are not a reason to postpone the first housing
milestone. They are the complete companion set for 17 provisionally mapped
empty-lot scenes, not the minimum needed to enter one lot with one fixture.

The smallest useful path is:

1. Route the already-built housing service through the gateway and use one
   already-loading scene as a bootstrap lot (R). Keep this clearly marked as a
   development lot, not a retail lot.
2. Use one fixture whose model chain is fully present: the capture-backed
   `hsg_block_01` chain, fixture definition 34, item definition 7466, model
   1926.
3. Accept enter, decorate/edit, place one owned fixture, pick it up, leave, and
   re-enter after a save. Do not include house lists, visits, directory,
   customization, upkeep, farms, or move/save in that first slice.
4. Make `hsg_emptylot_seaside_beach_01` the first real empty-lot scene. It has
   a captured client entry and all nine `.gcnk` terrain chunks. It is missing
   nine `.gck2` alternates and 48 of 64 `.dds` tile textures.
5. Only expand to the other scenes after the Seaside scene passes a native
   test. Six of the 17 provisional scenes have missing terrain geometry and
   should not be presented as enabled lots.

## Evidence boundary

This note uses:

- `notes/Build Guides/Gameplay/Housing.md`, especially the client senders and
  handlers in "Entering, visiting and leaving", "Decorate mode and placement",
  "Data and formats", and "Network".
- `notes/Research/Pieces/022 House loading and visiting.md` and
  `lab/reports/piece-022-20261009T025254Z/` for the preserved client capture.
- `lab/references/OpenSourceFreeRealms/ofrserver/Packets/p2.pcap`, SHA-256
  `65ac0801bf49fa9af4ee0bc95568b751392ec4d594fe6ac4e89605f9ee0d0108`.
- The client extraction catalog and the pinned community manifest through
  `scripts/check-asset-delivery-scenes.py`.
- `notes/Research/Housing Core.md` for the earlier 17-scene asset pass.

The capture is a real client session preserved by the OpenSourceFreeRealms
community. The capture proves that the client accepted these housing messages;
it does not prove that the endpoint was the original 2014 production server.
No captured player name, house name, chat text, or player-chosen text is
reproduced here.

Original packet facts, client reader/writer addresses, and capture values are
not interchangeable with our server policy. Rules supplied by this project
rather than recovered from the client are marked `(R)`.

## Client requests housing needs

Housing is application family 127. The family is tunneled as a gateway
application packet; its inner message begins with family `u16 = 127`, then a
`u16` subtype. The client-to-server senders and server-to-client handlers are
listed in the Housing guide.

### Entry and exit

| Subtype | Direction | Body | Client role | Minimum slice |
|---:|---|---|---|---|
| 23 | C>S | empty | `House.ShowingHouses` grant/list request | yes, for the My Houses flow |
| 19 | C>S | `u64 house guid, i32, u8` | enter the selected lot | yes |
| 31 | S>C | begin-zoning record | ordinary scene load to the lot scene | yes |
| 45 | S>C | two bools, `i32`, `PlayerHousingInstanceInfo` | housing zone data and in-instance state | yes |
| 28 | S>C | house instance record | creates/replaces the client house object | guide-required; not present in the captured fragment |
| 44 | S>C | edit byte, word, flags, counts, score | house info / edit confirmation | yes |
| 43 | S>C | counted fixture lists | decorate-bar item list | optional for entry; useful for placement UI |
| 10 | C>S | empty | leave the house | yes |

The minimum entry sequence is therefore:

`127/23 -> 127/19 -> scene 31 -> 127/45 -> 127/44`

The Housing guide also requires `127/28` before fixture messages. The captured
fragment below does not contain a decoded 127/28, yet the client accepted the
fixture sequence. Treat the need for 28 on our reply path as an unresolved
client-version/order question, not as something to silently remove. Keep it in
the first implementation and test it explicitly.

### Decorating and moving one fixture

| Subtype | Direction | Body | Minimum slice |
|---:|---|---|---|
| 6 | C>S | `u8` | yes, turn edit mode on/off |
| 44 | S>C | house info | yes, confirms edit mode |
| 1 | C>S | `u32, u32, u32, u8` | yes, request a fixture asset/selection |
| 42 | S>C | selection echo, fixture definition/model/item data | yes |
| 2 | C>S | `u32`, two 16-byte groups, `u32` | yes, commit one placement |
| 40 | S>C | fixture record | yes |
| 3 | C>S | `u64 fixture guid` | yes, remove the fixture |
| 41 | S>C | `u64 fixture guid` | yes |
| 5 | C>S | guid, two transform groups, nested block | no for place/remove; unresolved message |
| 51 | S>C | guid and transform | no for place/remove; paired with 5 |

The first slice should support place and pick-up. `127/5` (`SaveFixture`)
remains unresolved in the current service: its nested block has no recovered
fixture identity or proven position, so it returns an explicit unresolved
request rather than inventing a transform. Do not use `127/5` as the first
acceptance milestone.

### Later housing surface

These are real requests but outside the first slice:

- `127/14`, `127/15`, `127/16`, `127/21`: player-house lists, invites,
  decline, and neighbor visits.
- `127/9`, `127/17`, `127/22`: names and locking.
- `127/11`, `127/12`, `127/13`: lock, flora, and pet-autospawn toggles.
- `127/7`: upkeep.
- `127/20`: preview by item definition.
- `127/4`: pick up all fixtures.
- `127/59`, `127/60`: group customization.
- `127/53`: placement denial; `127/46`: housing errors.
- `127/54`, `127/55`, `127/56`, `127/57`: browser, explosion, clear, and
  server version.
- Rating/directory requests on family 174 (6, 7, 8, 12, 16, 20) and report
  family 68 (3). Their reply layouts are not recovered and are not needed for
  a private lot slice.

## Captured client flow

The preserved capture provides a stronger sequence than the current
reconstruction for one lot:

| Frame | Direction | Message |
|---:|---|---|
| 27110 | C>S | `127/23` grant request, exact bytes `7f001700` |
| 27195 | C>S | `127/19` enter, exact bytes `7f001300da102db86931f5070000000000` |
| 27311 | S>C | inner 31 begin-zoning |
| 29881 | S>C | `127/45` zone data, first flag set |
| 31311 | S>C | `127/44` and a second `127/45`, first flag clear |
| 31507 onward | S>C | nine complete `127/40` fixture updates |
| 31494 onward | C>S/S>C | the client's `127/1 -> 127/42 -> 127/2 -> 127/40` fixture restoration cycles |
| 32782 onward | C>S/S>C | `127/5 -> 127/51` and `127/3 -> 127/41` |
| 33569 | C>S | `127/6` set-edit-mode |

The begin-zoning record decoded to:

- scene: `hsg_emptylot_seaside_beach_01`
- zone type: 2
- zone id: 1
- geometry id: 214
- position: `(440.63199, -0.071, 432.80099, 1.0)`
- rotation: `(-0.99997407, 0.0, -0.00720356, 0.0)`
- sky: `sky_seaside24.xml`
- tutorial: false
- update radius: 0.0
- override update radius: true
- wait for zone ready: false
- begin-zoning SHA-256:
  `f3b03575443e99b5d3bd15f24752eedeec258375ac0d995a876a78cb5a435f07`

The two zone-data bodies are 118 bytes each. The first has the second flag
set; the second has it clear. The service currently emits one zone-data
message. The two-message capture is the better first test order. The exact
meaning of the first flag is not established.

The nine fixture updates all bind to the instance guid in the enter request.
This is evidence that the captured fixtures belong to the entered house and
that the instance identity, not a guessed property id, is the binding key.

The capture does not contain a decoded `127/28`, `127/39`, or `127/43` in this
fragment. That does not justify deleting those handlers from the client-side
model. It means the first native test must distinguish:

- what the current client requires to enter an empty lot;
- what our service adds from the Housing guide;
- which extra state messages the real client silently tolerates.

## Asset truth

### What "missing" means here

The check compared every file named by the pinned 162,652-row community
manifest against `lab/extracted/client-assets/catalog.tsv`, accepting only a
name plus exact CRC32 match. It then searched the preserved `lab/references`
tree for an exact-name file with a matching CRC. It did not use the network.

For the 17 provisional housing scenes:

| Type | Listed | Present in extracted client packs | Missing |
|---|---:|---:|---:|
| `.gzne` scene definitions | 17 | 17 | 0 |
| `*Areas.xml` | 10 | 10 | 0 |
| `.txt` (tile info and related) | 16 | 16 | 0 |
| `.gcnk` terrain chunks | 202 | 135 | 67 |
| `.gck2` alternate chunks | 202 | 0 | 202 |
| `.dds` tile textures | 1,325 | 321 | 1,004 |

The unique missing total is 1,273. The report has 1,547 per-lot rows because
four scene names are reused by more than one provisional property row. In
particular, `blackspore_01`, `briarwood_01`, `snowhill_01`, and
`shrouded_gloam_01` are duplicated across property rows.

Across the wider `hsg_emptylot` namespace, the manifest lists 2,248 names and
the extracted client catalog contains 616 CRC-matching names. The 616 are not
all of the 17-scene set; they include other housing scenes and related files.

All 1,273 missing names have zero matching loose sources under
`lab/references`. They are genuinely absent from the verified local sources;
this is not an extraction-path naming accident. The current asset checker
passed and reproduced the 1,273 / 1,547 totals on 2026-10-17.

### Where the missing files are

| Scene | Missing | `.gck2` | `.gcnk` | `.dds` | Terrain status |
|---|---:|---:|---:|---:|---|
| `hsg_emptylot_seaside_beach_01` | 57 | 9 | 0 | 48 | all 9 GCNK present; captured entry |
| `hsg_emptylot_snowhill_01` | 57 | 9 | 0 | 48 | all 9 GCNK present |
| `hsg_emptylot_island_01` | 58 | 9 | 0 | 49 | all 9 GCNK present |
| `hsg_emptylot_blackspore_01` | 61 | 9 | 0 | 52 | all 9 GCNK present |
| `hsg_emptylot_boat_01` | 61 | 9 | 0 | 52 | all 9 GCNK present |
| `hsg_emptylot_briarwood_01` | 61 | 9 | 0 | 52 | all 9 GCNK present |
| `hsg_emptylot_bw_towers_01` | 60 | 16 | 0 | 44 | all GCNK present |
| `hsg_emptylot_briarfalls_01` | 71 | 16 | 0 | 55 | all GCNK present |
| `hsg_emptylot_snowdays_01` | 64 | 16 | 0 | 48 | all GCNK present |
| `hsg_emptylot_vale_steam_01` | 65 | 16 | 0 | 49 | all GCNK present |
| `hsg_emptylot_wugachug_01` | 71 | 16 | 0 | 55 | all GCNK present |
| `hsg_emptylot_bog_shore_01` | 96 | 16 | 16 | 64 | no GCNK |
| `hsg_emptylot_gl_staircase_01` | 82 | 9 | 9 | 64 | no GCNK |
| `hsg_emptylot_sh_batcave_01` | 187 | 9 | 9 | 169 | no GCNK |
| `hsg_emptylot_sh_crystalmines_01` | 109 | 9 | 9 | 91 | no GCNK |
| `hsg_emptylot_shrouded_gloam_01` | 95 | 16 | 15 | 64 | 1 of 16 GCNK present |
| `hsg_emptylot_wilds_halflot_01` | 18 | 9 | 9 | 0 | no GCNK and no listed DDS |

The 11 scenes with complete GCNK are the only reasonable candidates for the
first native lot test. The six with missing GCNK have a structural geometry
gap even though their `.gzne` is present.

### What each missing type can block

- `.gzne`: none of the 17 scenes is missing its definition. The client can
  resolve the scene name, unless the file content itself is a header-only
  stub.
- `.gcnk`: the always-requested terrain geometry. `ChunkBuildAssetPaths`
  formats and forwards `.gcnk` unconditionally (`00d4eb30`). Missing chunks
  mean missing terrain, not merely missing decoration.
- `.gck2`: an alternate chunk format. `00d4eb30` requests it only when the
  alternate-format flag is set; the primary `.gcnk` path is still requested.
  The missing `.gck2` set is therefore a conditional/visual gap, not proven to
  be a hard entry blocker. Its exact switch and payload semantics are still
  unrecovered.
- `.dds`: tile textures. The client's `Tile::CreateAll` (`00d4ba50`) checks
  each tile texture handle and returns 0 on a missing texture or missing
  fallback. A 404 can fail tile creation unless the client's fallback path is
  active. The exact fallback condition is unrecovered; test it rather than
  treating a texture 404 as harmless.
- `Areas.xml` and `TileInfo.txt`: present for the 17 scenes. They are audio
  area and minimap/tile metadata, not the terrain chunk itself.

The client also has runtime fallback code (`RuntimeTerrainFallback`,
`fallback_%s_%d_%d`, `InstallFallbackTerrainResource`), but no fallback asset
name or required state was recovered. It is not safe to invent one.

### Delivery status

The current `AssetDelivery` serves only CRC-verified local sources. The fresh
checker verified delivery for `.gzne`, `Areas.xml`, `.gcnk`, `.dds`, `.txt`,
and `.gck2` where a verified source exists. A missing name returns no response
(HTTP 404 behavior). There is no verified local substitute for any of the
1,273 missing files.

## Recommended smallest playable slice

### Slice 0: bootstrap lot

Use one existing, already-loading scene as the lot background and label the
mapping `(R)`. `sg_random_encounter_clearing` is the best bootstrap:

- it has 4 of 4 `.gcnk` chunks and 23 of 25 `.dds` files present;
- it already has a captured zone transition and a prior native acceptance pass
  for an encounter scene;
- it is small and isolated compared with `FabledRealms`.

It is not a retail empty lot. Do not put it in the player-facing lot
catalogue as a recovered property.

The absolute smallest protocol slice is entry and exit with zero fixtures:

`127/23 -> 127/19 -> 31 -> 127/45 -> 127/44 -> 127/10 -> 31`

Make that native gate green before adding the block. If it fails, the fixture
layer is irrelevant.

Use this bootstrap sequence:

1. Give the test character the starter property and seed item definition 7466
   as one owned instance.
2. Load the `FixtureConfig` row that pairs item 7466 with fixture definition
   34 and model 1926. The model chain is fully present:
   `hsg_block_01.adr`, `.dma`, `.dsk`, `.cdt`, `.dds`, and
   `hsg_block_01_lod0.dme` all have CRC-matching catalog entries.
3. Route `127/23` to a house-list response, then `127/19` for the single test
   house. The list response is needed for a UI-driven flow and is not present
   in the current service.
4. On enter, emit the scene change first, then zone data, instance data, house
   info, and the owned-item list. Preserve the captured two-zone-data order
   as an experimental option; the current one-message order remains `(R)`.
5. On `127/6`, answer `127/44`. On `127/1`, answer `127/42`. On `127/2`,
   validate and stage the inventory consumption, then answer `127/40` and
   `127/44`. On `127/3`, return the item and answer `127/41` and `127/44`.
6. On `127/10`, return to the prior world scene and persist the fixture.

This slice needs no new scene asset recovery. It proves the protocol, the
inventory/fixture conservation rule, and the client's decorate interaction
before the asset question is mixed in.

### Slice 1: first real empty lot

Promote `hsg_emptylot_seaside_beach_01` for the first retail-looking lot. It is
the only scene with both:

- an exact recovered client entry in the preserved capture; and
- all nine `.gcnk` terrain chunks present.

Use the captured begin-zoning values in the "Captured client flow" section,
not a Legacy build-box guess. The scene has 16 of 64 tile textures and none of
the nine `.gck2` alternates. Test the native client against those 404s first.
If tile creation fails, recover the nine `.gck2` and 48 `.dds` by exact
manifest name and CRC, or implement and verify a real fallback. Do not serve a
replacement texture under the original name: `AssetDelivery` intentionally
rejects CRC/substitution mismatches.

Do not enable property rows for the six scenes with missing GCNK until their
terrain is recovered or a native path proves that the absent chunks are outside
the loaded area. The current `houses.json` marks several of those scenes
enabled; its `enabled` flag checks content tags, not asset availability. That
is a content trap, not evidence that the scene is playable.

### Deferred

- `127/5 -> 127/51` save/move: nested layout and identity remain unresolved.
- House lists, visits, invitations, directory, ratings, and reports.
- Naming, lock, flora, pet autospawn, upkeep, customization, interactive
  furniture, and farms.
- All 24 empty-lot definitions and the full 1,273-file asset recovery.

## Current integration blockers

The modules are registered but not in the production route:

- `gateway.rs::application_handler` has no family 127 branch. Its final
  fall-through sends family 127 to `gameplay.handle_in_world`, which has no
  housing branch; a probe from the prior housing review saw zero replies and
  `changed=false`.
- `Gameplay` has no housing service field. `HousingService` has a tested
  callable API, but no production caller.
- `game_store.rs` has an optional housing payload and load/save helpers, but
  the gameplay snapshot does not carry the service state.
- `HousingContent::bundled()` exists but `osfr-gateway` does not load
  `houses.json` or construct a `HousingService` for a login.
- The client list path (`127/23`, `127/14`, `127/39`) is not implemented by
  the service, so a UI-driven My Houses screen cannot yet discover the test
  lot.
- `127/5` remains intentionally unresolved. A place/pickup slice can avoid
  it, but a complete move feature cannot.
- The current 21 enabled property rows are provisional `(R)` icon-name
  matches. The capture identifies a scene, not a property-to-scene table.
- The captured begin-zoning `zone id` 1 is not a unique property key; the
  current property-1 row points at Blackspore while the capture scene is
  Seaside. Do not "fix" that row by name association alone. Use the captured
  Seaside scene as the first native target, then establish whether the
  property row actually belongs to it.

## Acceptance gates for the first slice

1. Unit/service gates: one owner gets one starter lot; place consumes exactly
   one owned instance; pickup returns exactly one instance; every refusal
   leaves lot, inventory, and selection state unchanged.
2. Native gate: My Houses -> enter bootstrap lot -> edit -> place the block ->
   see it -> pick it up -> leave -> restart -> enter again. Confirm the saved
   fixture count and inventory count.
3. Asset gate: run the scene through the current `AssetDelivery`; record every
   404 by exact name. A 404 must be treated as a failed asset gate, not a
   silent success, unless the client's fallback is observed and documented.
4. Only after the bootstrap slice passes, repeat the native gate on
   `hsg_emptylot_seaside_beach_01` with the captured scene transition.

## Open gaps

- No recovered property-to-scene table. The capture is one scene observation,
  not a mapping for all 31 Legacy rows.
- `127/28` necessity/order conflicts with the captured fragment; test both
  the guide-required and capture-minimal sequences.
- The exact `.gck2` alternate flag, `.dds` fallback behavior, and
  `Tile::CreateAll` fallback condition remain unrecovered.
- The missing 67 `.gcnk` are concentrated in six scenes and are likely
  structural; no native proof of fallback exists.
- The `127/2` placement selector catalogue, retail fixture definition table,
  per-lot limits, collision/snap/floor semantics, and `127/5` nested body
  remain open.
- The captured flow is community-server evidence, not proof of the original
  2014 server's reply policy.

## Evidence commands

Run from `project/`:

```text
python -I scripts/check-asset-delivery-scenes.py
```

Result on 2026-10-17:

```text
checks=passed
enabled_lots=21
unique_scenes=17
unique_missing_files=1273
lot_file_entries=1547
missing=.dds:1004 .gck2:202 .gcnk:67
```

The capture decode and byte vectors are recorded in
`lab/reports/piece-022-20261009T025254Z/checker-report.json` and
`entrance-p2.json`. The current asset report is
`lab/content/housing-assets/missing.json`.

Cross-links: [[Housing Core]], [[Housing Restoration]],
[[Housing Service Integration]], [[022 House loading and visiting]],
[[078 Zone transitions]], [[../Build Guides/Gameplay/Housing]].
