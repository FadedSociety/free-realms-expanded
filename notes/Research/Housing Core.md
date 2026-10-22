# Housing core (W24)

Status: service slice implemented and focused tests pass. This is not gateway integration or native
client acceptance. The client guide is the specification; Legacy is a reference for content shape
and behavior only. Every server policy below that the client does not establish is marked (R).

## Client evidence used

Source of truth: [[../Build Guides/Gameplay/Housing]]. It explicitly excludes OSFR packet captures
and server JSON from evidence and separates client findings from reconstructed server rules.

- Entry: the client sends 127/19 with the chosen house guid (`House.Enter`, `00ac3fe0`). Success
  requires an ordinary begin-zoning message to an `hsg_emptylot_*` scene before housing data
  (`Housing.md`, lines 46-51). The client has no property-to-scene, spawn or build-box table.
- Entry packets: client readers are 127/45 at `00ab6c80`, 127/28 at `00ac9dd0`, and 127/44 at
  `00ab8ee0`. The client requires 127/28 before fixture packets. The guide describes
  begin-zoning -> 45 -> 28 -> 44 as the expected sequence, but explicitly labels the exact server
  order (apart from 28-before-fixtures) reconstructed. This service emits the outer scene change
  first and then replies 45, 28, 44, 43; the 43 position and complete ordering remain (R).
- Edit and placement: the client sends edit toggle 127/6 (`00ac4160`); 127/44 updates the confirmed
  edit flag. Request 127/1 has body `u32,u32,u32,u8` (`00ac4200`); 127/2 has `u32`, two 16-byte
  groups and a final `u32` (`00abffb0` / writer `00abc9a0`). The Housing Placement Selection
  Identity Review confirms the first 127/2 word is a client selection id echoed through 127/42,
  distinct from the fixture and item ids. This does not prove server ownership authorization or
  identify the selection catalogue; see [[../Reviews/Housing Placement Selection Identity Review]].
  The service therefore binds opaque selection ids only to live owned inventory instances and
  rejects unresolved ids with 127/53.
- Move and return: client 127/5 writer `00abf620` sends a fixture guid, transform groups, a dword
  and nested data. The native disassembly was checked with `python -I scripts/native_map/nmq.py dis
  00abf620`; it shows the writer header call, a field at object offset `+0x10`, two four-dword
  writes from `+0x20` and `+0x30`, a scalar at `+0x40`, then trailing serialization. Field semantics
  beyond the guide's client read remain unresolved; the nested tail is ignored by this service.
  Client 127/3 carries a fixture guid (`00ac42d0`), 127/4 is empty (`00ac8680`), 127/10 is empty
  (`00ac4b80`), and 127/51 applies a server transform (`00ab8dc0`). The full 127/51 body is not
  recovered here.
- Client resources: `Housing.md` lines 94-102 lists `HouseNames.txt`, `HousingNames.txt`,
  `HousingPlacementData.txt`, `HousingPlacementAssociations.txt` and `FactoryPlots.txt`. These
  provide naming/UI, developer placement samples and farm plots, not a lot/zone/build-box table.
  Asset names include `hsg_*` models and `hsg_emptylot_*` scenes, but no property mapping.
- Limits: the client receives a `HousingMaxFixtures` denial string, but no numeric cap per lot is
  recovered; permission level numbers and original server validation are also unknown
  (`Housing.md` lines 188-205 and 230-249).

## Legacy reference and content

Consulted as reference only:

- `lab/references/FreeRealms-Legacy/src/Sanctuary.Game/Resources/Definitions/HouseDefinition.cs`
  supplies the data shape: Id, NameId, ZoneId, Icon, spawn position/rotation and build areas.
- `lab/references/FreeRealms-Legacy/src/Resources/Houses.json` supplies 31 starting rows. Its SHA256
  is `854b2d3f009b916a928f66d57a42e2bf685dcc853e898d149580eefd26196d3c`; the pinned checkout is
  `9e082025391177d5a33ee18c714ddd06b457a695` (AGPL-3.0). The rows in `data/content/houses.json`
  retain a per-row Legacy source tag. No Legacy source code was copied.
- `Sanctuary.Gateway/Handlers/BaseHousingPacketHandler.cs` routes only Legacy edit-mode and enter
  handlers, with other housing requests unimplemented. `Sanctuary.Packet/BaseHousingPacket.cs`
  frames base opcode 127 plus a subtype. Neither file is evidence of retail server behavior; no
  Legacy handler behavior was ported.

The client tables do not identify which Legacy property row maps to which client lot scene. Thus all
31 content rows have `scene: null` and `enabled: false`; entering one fails closed with
`HouseSceneUnavailable`. The Legacy geometry is only a provisional starting point, not recovered
client build-box data. Rows with inverted Legacy box components are normalized per component and
tagged with that reconstructed correction.

## Reconstructed policy (R) in this slice

- Property id 25, the Wilds Condo (name id 296), is the starter lot type (L030, 2026-10-22);
  `grant_starter_lot` grants one per character and is idempotent per (owner, property). The wiki
  records it as every new player's starter from 2012-01-26
  (`Research/Pieces/021 Property acquisition.md:107-108`) and the client locale's decorating
  tutorial addresses "your new Wilds Condo"; the exe names no housing scene (IDA `strings` for
  `condo` and `hsg_` find nothing), so the server chooses it. A character who already holds the old
  starter (property 1) keeps it and gains the condo on its next 127/23. The 2012 rule named only
  Apartment owners; giving it to every earlier owner is (R).
- Visitor level 1, edit threshold 3, edit-mode enforcement, lock behavior, and all lot permission
  numbering are server policy. The Legacy packet model is only a reference for level 3.
- `127/44`'s fixed local codec fields `unused = 10` and `is_flora_allowed = true` are retained as
  reconstructed defaults from the earlier service slice; the client guide does not establish those
  values. They need client acceptance or better evidence before being treated as retail values.
- The provisional lot cap is 9999 fixtures and zero landmarks. The client supplies no per-lot cap
  table; 127/53 strings are reconstructed except for the packet's recovered one-string shape.
- Build-box containment and finite/range checks are reconstructed server validation. The bundled
  Legacy boxes are not client verified.
- `127/2` selection ids are never treated as item ids. `HousingService::offer` requires a current
  owned inventory instance with count 1 and a configured fixture row. Placement revalidates the
  binding, permissions and edit mode, stages the lot, consumes through `Inventory`, then commits.
  Pickup returns through `Inventory`; pickup-all stages all lot changes and applies an atomic return
  batch before committing. The `FixtureConfig::capture_defaults` table remains an explicitly
  reconstructed test fallback from an older packet-derived artifact; it is used only to supply
  test configuration rows, not as client-table truth or to infer selection ownership.
- Move, place, pickup, return-all and edit mode require the reconstructed confirmed edit session.
  The typed fixture cap/build-box checks and permission failures are handled before any inventory
  mutation.

## Implementation and persistence

- `src/housing_service.rs` owns source-tagged lot content validation, starter grants, visit/edit
  sessions, selection-to-owned-instance binding, placement/move/pickup/return-all and typed scene
  transition events. Enter returns `HousingZoneChange::EnterHouse`; leave returns
  `ReturnToPreviousZone`. The gateway must perform that outer zone transition before sending housing
  replies.
- `game_store.rs` retains housing as an optional payload in the canonical save envelope and exposes
  typed `load_housing_state` / `save_housing_state` helpers. Housing-only saves use schema 5; saves
  that also contain pet state use schema 6. Schema 4 -> 5 migration preserves housing payload and
  instance counters. `tests/save_schema.rs` covers the migration. The gameplay/gateway route is not
  wired, so live inventory and housing are not yet committed together by a caller.
- Focused checks: `tests/housing_service.rs` covers grant, enter ordering, placement, move, pickup,
  pickup-all, reload, cap, unresolved selector and visitor denial. `tests/housing.rs`,
  `tests/housing_native.rs` and `tests/save_schema.rs` are regression gates.

## Lead integration steps

1. Register `housing_service` in `lib.rs` and replace its temporary `#[path]` restoration include
   when the restoration module is registered.
2. Add gateway ingress for family 127. Apply `zone_change` before housing replies; do not serialize
   `HousingReply::Unrecovered` bodies. Persist service state when `ServiceOutcome.changed` is true.
3. Keep inventory and housing changes in one canonical save transaction when wiring the service.
   Use `HousingServiceState` with the game-store housing helpers or extend the shared snapshot shape
   in the lead-owned integration change.
4. Supply property-to-scene mappings and validated scene-derived spawn/build bounds before enabling
   any bundled lot. The reader-layout codecs for 127/42, 127/43, 127/45 and 127/51 now exist, but
   their reconstructed values still need native acceptance. Keep selector resolution fail-closed;
   the original catalogue owner remains unknown.
5. Replace the fixture config fallback with source-tagged fixture definitions, then run native
   client acceptance for entry, edit, owned-item placement, move and return.

## Open gaps

- No client lot/scene/spawn/build-box mapping; all bundled lots are disabled.
- No recovered source table that generates the client's full 127/1 selection catalogue. `offer`
  binds an explicit selector to one owned item; 127/1 now resolves only through that binding.
- The new 127/28, 127/42, 127/43, 127/45 and 127/51 codecs follow native reader field order, but
  unknown values, catalogue policy and native client acceptance remain open.
- Fixture-definition data, retail permission values, numeric caps, original validation, invite and
  directory behavior remain unknown or outside this task.
- No gateway registration, live save integration, native game run or native housing acceptance was
  performed.
- One pre-existing unused `Lot::instance_index` warning remains in `housing.rs`.

## Scenes and payloads (W32, 2026-10-13)

### Scene and bounds result

The search found no evidence to associate any of the 31 `houses.json` property IDs with a client
scene or client build bounds. All rows remain `scene: null` and `enabled: false`; their source tags
now separately name `client-scene-unmapped` and `client-build-bounds-unmapped`. No row is enabled by
semantic name matching, coordinate proximity, or the Legacy bounds.

Evidence inspected:

- The pinned `lab/references/osfr-asset-manifest/Assets_manifest.txt` lists 24
  `hsg_emptylot_*.gzne` names (including `hsg_emptylot_briarwood_01` and
  `hsg_emptylot_snowhill_01`) and 13 `hsg_emptylot_*Areas.xml` names. This is a community manifest;
  the candidate files did not match the scanned client-assets extraction by size and CRC, and the
  manifest supplies no property-ID association.
- The only extracted area XMLs located for this check were Briarwood, Merryvale and Snowhill
  community-pack outputs. Their sphere definitions are `Sound(Background)` or `Music(Background)`
  regions. They are audio volumes, not fixture build bounds. The named placement text resources
  (`HousingPlacementData.txt` and `HousingPlacementAssociations.txt`) were not present as extracted
  client files in the inspected tree.
- Native `00aab1a0` uses the housing processor table at `+0x28b0` as a 16-bucket,
  case-insensitive fixture-group string lookup; `00aae6b0` performs a two-string lookup and checks
  its result count. Neither function maps a property to a scene or returns build-box geometry.
- The client guide's `HouseNames.txt`, `HousingNames.txt`, `HousingPlacementData.txt`,
  `HousingPlacementAssociations.txt` and `FactoryPlots.txt` inventory is retained at
  `notes/Build Guides/Gameplay/Housing.md#Data-and-formats`; names and fixture placement samples do
  not establish lot scenes or bounds.

Consequently the existing Legacy six-float boxes remain provisional policy only. The `127/28`
builder transcribes each local `[min_x, min_y, min_z, max_x, max_y, max_z]` box into the client
reader's two float4 records by writing `w = 0` (R). This makes synthetic enabled test content
serializable; it does not make those boxes client-derived or safe for a real lot.

### Reader layouts and service values

The new `housing_native` codecs use these client reader anchors. Reader field order and primitive
widths below are recovered facts; each service value labeled R is a reconstructed placeholder or
policy, not an original server value.

- **127/28 instance data** — handler/body validator `00aca330` / `00aca2b0`, body reader
  `00ac9dd0`. It reads house and owner GUIDs, owner name, unknown u64, name ID/name, unknown word,
  fixture and landmark caps, counted keyed fixture records (`00ab8c50`, record reader `00ab6f40`),
  counted keyed permission records (`00ab8d10`, nested reader `00ab3950`), lock/flora flags, current
  counts, unknown bool/word, counted build areas (`00ab7250`, two float4s each), icon/preview flags,
  score, unknown u64, members flag, two strings, unknown bool, then an optional factory record. The
  service emits full empty/display fields and factory marker false (R); current fixtures are encoded
  as full records, while list keys are stable ordinals (R). A true factory marker fails closed.
- **127/42 fixture asset** — handler `00ac7f40`, reader `00ac1f50`. The body is two u32 words,
  fixture-definition tail `00abeb40`, two length-prefixed spans, two u32 words, another span, a
  counted u64 GUID list (`00aafa90`), two u32 words and a bool. The selection review establishes
  that the 127/42 second word echoes the selector. Unknown words, strings, list and flag use empty
  or zero defaults (R), and the definition comes from the current reconstructed fixture config.
- **127/43 item list** — handler `00ac9bd0`, parser `00ac9a90`; it reads the counted entry list
  (`00ab3620`, each nested `00aafd30`), fixture-definition list (`00abf760`), and counted u32 list
  (`00aafb50`). The service projects known owned furniture instances and unique reconstructed
  definitions into the first two lists; instance-ID projection and the empty final list are R.
  This packet shape has no proven client selector-to-inventory mapping. The existing `offer` binding
  remains that authority seam; a 127/1 selector now returns 127/42 only when that explicit binding
  still points to a single currently owned instance.
- **127/45 zone data** — handler `00ac36c0`, reader `00ab6c80`; it consumes two bools, a u32 word,
  then `PlayerHousingInstanceInfo` via `00aafe10`. That nested reader consumes two GUID-width
  pairs, name ID, two strings, two words, a two-word pair, one word, three bools, three strings,
  float, word, two bools, word and a final two-word pair. The service sends the lot owner/instance/name/
  icon/count and lock state; flags, display strings, scores, ratings, timestamps and other unknown
  values use R defaults.
- **127/51 transform update** — handler `00ab8dc0`, nested reader `00ab2d10`. It reads two u32
  words (the client record's 64-bit GUID pair), then two float4 blocks. The service sends the saved
  fixture GUID, `[x,y,z,1]`, and its stored rotation. The semantic projection and position w value
  are R; all values must be finite under the local codec.
- **127/44 house info** remains the existing fixed writer. The local `unused = 10` and flora=true
  values remain R; the client guide does not recover their server values.

The 127/1 client writer `00abcb90` sends three u32s and one byte. Existing capture/review evidence
ties its first selector to the second 127/42 word; it does not identify the selector's catalogue or
prove that selector IDs are inventory/item IDs. The service therefore preserves the selector as an
opaque key and refuses requests with no `offer(actor, selector, owned_instance, stock)` binding.

### Verification and limits

`tests/housing_service.rs` now checks the synthetic test lot's opcode-31 zoning frame followed by
exact complete inner bytes for 127/45, 127/28 and 127/44, then the empty 127/43 list. It also checks
that bundled rows remain disabled and that selectors resolve through `offer`. `tests/housing_native.rs`
round-trips synthetic records for 127/28, /42, /43, /45 and /51. These are codec and service tests,
not native-client acceptance. No real lot scene/bounds mapping, server payload capture, gateway
registration, original selector catalogue, factory payload, or retail unknown/default values were
recovered in W32.

## Enabled lots (R) (W39, 2026-10-13)

The client still has no property-to-scene table. For this provisional content pass, each Legacy
`Icon` value was joined to the supplied client's `Resources/Images/Images.txt` to get its icon
asset name. Matching the region/product words in that name to a community scene filename is a
reconstructed association (R), not a recovered client rule. `HouseNames.txt`, `HousingNames.txt`,
`HousingPlacementData.txt`, `HousingPlacementAssociations.txt` and `FactoryPlots.txt` do not add a
property-to-scene association. `Assets_manifest.txt` names candidate scenes but does not associate
them with properties.

| Property | Client icon asset name | Scene | State / reason |
|---:|---|---|---|
| 1 | `icon_hsg_mkt_blackspore_emptylot_64.dds` | `hsg_emptylot_blackspore_01` | enabled (R) |
| 2 | `icon_hsg_mkt_yachtlot_emptylot_128.dds` | `hsg_emptylot_boat_01` | enabled (R, yacht lot -> boat scene) |
| 3 | `icon_hsg_mkt_bog_shore_emptylot_64.dds` | `hsg_emptylot_bog_shore_01` | enabled (R) |
| 4 | `icon_hsg_mkt_briarfalls_emptylot_64.dds` | `hsg_emptylot_briarfalls_01` | enabled (R) |
| 5 | `icon_hsg_mkt_briarwood_emptylot_64.dds` | `hsg_emptylot_briarwood_01` | enabled (R) |
| 6 | `icon_hsg_mkt_briarwoodtowers_emptylot_64.dds` | `hsg_emptylot_bw_towers_01` | enabled (R) |
| 7 | `icon_hsg_mkt_generic_emptylot_64.dds` | — | disabled: generic icon has no unique region scene |
| 8 | `icon_hsg_mkt_gl_staircase_emptylot_64.dds` | `hsg_emptylot_gl_staircase_01` | enabled (R) |
| 9 | `icon_hsg_mkt_shrouded_glade_emptylot_64.dds` | `hsg_emptylot_shrouded_gloam_01` | enabled (R, name-to-region match) |
| 10 | `icon_hsg_mkt_island_emptylot_64.dds` | `hsg_emptylot_island_01` | enabled (R) |
| 14 | `icon_hsg_mkt_seaside_emptylot_64.dds` | `hsg_emptylot_seaside_beach_01` | enabled (R, selected over cliffs scene) |
| 15 | `icon_hsg_mkt_batcave_emptylot_32.dds` | `hsg_emptylot_sh_batcave_01` | enabled (R) |
| 16 | `icon_hsg_mkt_crystalmines_emptylot_64.dds` | `hsg_emptylot_sh_crystalmines_01` | enabled (R) |
| 17 | `icon_hsg_mkt_shroudedgloam_lot_64.dds` | `hsg_emptylot_shrouded_gloam_01` | enabled (R) |
| 18 | `icon_hsg_mkt_snowdays_emptylot_64.dds` | `hsg_emptylot_snowdays_01` | enabled (R) |
| 19 | `icon_hsg_mkt_snowhill_emptylot_64.dds` | `hsg_emptylot_snowhill_01` | enabled (R) |
| 20 | `icon_hsg_mkt_vale_stream_emptylot_64.dds` | `hsg_emptylot_vale_steam_01` | enabled (R, icon/scene spelling differs) |
| 21 | `icon_hsg_mkt_wugachug_emptylot_64.dds` | `hsg_emptylot_wugachug_01` | enabled (R) |
| 22 | `icon_hsg_mkt_blackspore_emptylot_64.dds` | `hsg_emptylot_blackspore_01` | enabled (R) |
| 23 | `icon_hsg_mkt_generic_emptylot_64.dds` | — | disabled: generic icon has no unique region scene |
| 24 | `icon_house_apartment_64.dds` | — | disabled: no matching named community lot scene |
| 25 | `icon_hsg_mkt_hsg_hum_condo_w_lot_64.dds` | `hsg_hum_condo_w_lot` | enabled, starter (R, `client-manifest`: the icon is named after the scene; see below) |
| 26 | `icon_house_deluxe_64.dds` | — | disabled: no region-specific community lot scene |
| 27 | `icon_house_economy_64.dds` | — | disabled: no region-specific community lot scene |
| 28 | `icon_house_viphouse_64.dds` | — | disabled: no region-specific community lot scene |
| 29 | `icon_hsg_mkt_snowhill_lodge_64.dds` | `hsg_emptylot_snowhill_01` | enabled (R, region match) |
| 30 | `icon_hsg_mkt_rumbledome_emptylot_128.dds` | — | disabled: no matching Rumbledome scene |
| 31 | `icon_hsg_mkt_sunstonevalley_emptylot_64.dds` | — | disabled: no matching Sunstone Valley scene |
| 32 | `icon_hsg_mkt_dwarfdam_emptylot_64.dds` | — | disabled: no matching Dwarf Dam scene |
| 33 | `icon_farming_briarwood_lot_01_64.dds` | `hsg_emptylot_briarwood_01` | enabled (R) |
| 34 | `icon_farming_wilds_lot_64.dds` | `hsg_emptylot_wilds_halflot_01` | enabled (R, selected over rapids scene) |

All enabled rows use `bounds_source: "legacy (R)"` and `scene_source: "community-manifest (R)"`,
except property 25 (`client-manifest (R)`); each retains its Legacy six-float box and spawn values.
Property 25 is the starter. Scene `zone_type`, sky, unknown value, geometry ID and other
transition fields use the service test defaults (R); only the scene name was matched to a manifest
entry. These defaults and the bounds have not passed native client acceptance. Nine rows without a
plausible unique candidate remain disabled and carry a reason in JSON.

Property 25's earlier search looked only at `hsg_emptylot_*` names. Its icon names the scene
`hsg_hum_condo_w_lot`, which ships in the working client: `Assets_manifest.txt:72474`
(`hsg_hum_condo_w_lot.gzne.z`, CRC 274745638) and `:146197` (`.map`), 183 manifest entries in all;
the scene body is `Assets_004.pack` entry 2685 (`catalog.tsv:72023`, CRC `10604926`, the same
value). The extracted body is `GZNE` version 3 and holds only terrain material names (`sg_dirt`,
`sg_grass`, `bw_grass_darkvines` ...), no sky or geometry id, and the toolkit has no GZNE field
reader, so its transition fields keep the defaults above (R). `hsg_hum_condo.gzne` (84 raw bytes) is
a header-only stub and is not used.

### Asset delivery finding

W39 rechecked manifest names through `catalog.tsv`'s `extracted_relative_path` and CRC32. This
supersedes W32's broad statement that candidate scene/area files did not match the client
extraction: all 24 manifest `.gzne` scene bodies and all 13 `*Areas.xml` bodies are present in the
extracted client packs and match the manifest CRC. The manifest length is a compressed/wire length,
while catalog `bytes` is the raw extracted length; compare CRCs, not those two lengths. For example,
Briarwood's scene has manifest length 637 and raw length 1,971, with matching CRC.

The run launcher serves HTTP requests through `AssetDelivery` (`scripts/run-local-baseline.py:146-191`).
`scripts/asset_delivery.py:29-31,63-91` supports only `.z` and `.gcnk`; `.gzne`, `.gck2`,
`.Areas.xml`, `.dds`, `.txt` and `.gcnk` assets without a CRC-matched source are not served. A
read-only `AssetDelivery.get`
probe returned no scene/area/cache source for Briarwood `.gzne`, `.gck2` or `Areas.xml`, while a
CRC-matched Briarwood `.gcnk` returned successfully. The referenced `asset-cache` and `http-raw`
indices contain zero direct `hsg_emptylot` files. No assets were downloaded.

For the 17 unique enabled scene bundles, the exact manifest-listed companion counts versus verified
raw files in the supplied client extraction are:

| Type | Present / listed | Missing from extracted client packs |
|---|---:|---:|
| `.gzne` scene definitions | 17 / 17 | 0 |
| `*Areas.xml` | 10 / 10 | 0 |
| `.txt` | 16 / 16 | 0 |
| `.gcnk` | 135 / 202 | 67 |
| `.gck2` | 0 / 202 | 202 |
| `.dds` | 321 / 1,325 | 1,004 |

The missing set is the rows with those scene-name prefixes in the pinned community manifest that
have no matching raw CRC entry in the extracted `catalog.tsv`; the manifest contains the exact
filenames. The current HTTP route can serve only the locally present `.gcnk` subset. Whether every
listed companion is loaded on housing entry is not established because no `.gzne` dependency graph
was decoded. The extracted `.gzne` files can be available to a client loading its local packs, but
the run HTTP server is not a complete housing asset-delivery path.

### Asset delivery v2 (W42)

`AssetDelivery` now indexes every safe filename in the active manifest after canonicalizing an
optional trailing `.z`. The request suffix still chooses raw versus native-zlib response. This
matches confirmed client `AssetReceived` at `0109b400` (query `nmq.py show 0109b400`,
`lab/native-map/run/map.sqlite`; decompilation at `lab/reports/native-assets-functions.c:144-146`)
and manifest CRC callbacks `0109bd00`/`0109bea0` (query `nmq.py show 0109bea0 0109bd00`,
`lab/native-map/run/map.sqlite`; accepted labels at `lab/native-map/run/native-map.tsv:84871-84872`).
Compressed bodies retain the verified `A1B2C3D4`/length/zlib envelope; raw responses must match
manifest length (R).
Raw bytes are checked against manifest/catalog CRC and recorded size, plus the indexed or computed
SHA-256 when a community reference supplies one. Only exact CRC matches from the local extraction
or files under `lab/references` are eligible (source discovery is R); no network fallback is used.

`python -I scripts/check-asset-delivery-scenes.py` passes `.gzne`, `Areas.xml`, `.gck2`, `.gcnk`,
`.dds`, `.txt`, and `.adr` delivery checks. It writes
`../lab/content/housing-assets/missing.json` from the pinned community manifest and the verified
catalog. The report groups 1,273 unique missing files across 21 provisional lots / 17 scenes
(1,547 per-lot entries): 202 `.gck2`, 67 `.gcnk`, and 1,004 `.dds`. All 24 scene definitions and
13 `Areas.xml` files remain CRC-matched in the extraction. None of the 1,273 missing files has an
exact CRC-matching loose source under `lab/references`; saved pack-range fragments are not decoded.
No files were downloaded. This is HTTP/source verification, not native-client acceptance, and the
provisional lot-to-scene joins above remain R.

L030 added property 25 (`hsg_hum_condo_w_lot`): 22 lots / 18 scenes, 1,350 unique missing files
(1,624 per-lot entries): 202 `.gck2`, 67 `.gcnk`, 1,081 `.dds`. The condo's 183 entries are all
CRC-present except 77 `.dds` (its `.gzne`, `.map`, `.txt`, 20 `.gck2`, 20 `.gcnk` and every
`.adr`/`.cdt`/`.dma`/`.dme` are present); none of the 77 has a loose community source. The old
starter (`hsg_emptylot_blackspore_01`) misses 9 `.gck2` and 52 `.dds`. The script ran with its
report redirected to a scratch file; rewriting `lab/content/housing-assets/missing.json` is the
lead's call. Whether the missing `.dds` show as untextured surfaces needs the native check.

