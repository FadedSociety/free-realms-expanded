# 073 Discoveries and exploration

Status: closed for review, 2026-10-09 04:49 UTC. Evidence label: **ORIGINAL client vocabulary and a
byte-exact captured wire payload over a RECONSTRUCTED proximity/once-only policy**. There is no
native discovery run and no reward grant; the module grants nothing.

Owned paths used:

| Path | Role |
| --- | --- |
| `prototype/osfr-tools/src/restoration/piece_073.rs` | definition decoder, name index, discovery evaluator |
| `prototype/osfr-tools/tests/piece_073.rs` | 11 owned vectors + `#[path]` include + private capture gate |
| `scripts/pieces/piece_073.py` | independent Python checker over the preserved inputs |
| `tests/pieces/test_piece_073.py` | 16 stdlib tests, including two drift-rejection cases |
| `lab/reports/piece-073-20261009T0449Z/` | logs, locale export, Rust vectors, evidence JSON, provenance |

No shared `lib.rs`, gameplay, gateway, Cargo manifest, launcher, content, state or native UI file was
touched, and no other item's module or interface was overwritten.

## What is recovered (ORIGINAL)

### The client's point-of-interest system and its two packets

`lab/decompiled/client/FreeRealms/pseudocode.c` contains the client's own packet constructors:
`PacketPointOfInterestDefinitionRequest` writes `0x38` (56) and `PacketPointOfInterestDefinitionReply`
writes `0x39` (57) into the packet header (`:1572906`, `:1572928`). The supplied server's
`LibSOE/SOE/BasePackets.cs:32-33` spells the same two ids, and the reply is sent when the client
announces it is ready (`Gateway.Login/LoginManager.cs:65-68`). The client also ships the
`ClientPointOfInterestManager`, `CurrentPOIDataSource` and `PointOfInterestDefinitionManager`
vtables (`:1573244`, `:1760981`, `:2426798`) and registers the data source name
`BaseClient.PointOfInterestData` (`:1573880`).

The shipped `client-ui/ScriptsBase.lua` defines both data sources the UI reads:

* `BaseClient.PointOfInterestData` with the 12 columns `Id, Name, ZoneID, X, y, z, Heading, IconId,
  NotificationType, SubName, BreadcrumbQuestId, AtlasName` (`:2966-2980`);
* `BaseClient.CurrentPOI` with `POIName, POIId, AreaId` (`:3083-3087`), whose update handler treats
  `1 < poiId < 11` as a zone change and calls `NotificationHandler:LoadingZone(poiId)` (`:5049-5053`);
* the map places one indicator per row from the same data source (`:31133-31134`).

### The definition wire shape, decoded from a real captured message

The recorded gateway oracle holds exactly one message with inner opcode 57
(`lab/captures/legacy-gateway-20261008T014620082859Z/messages/0009-ready.bin`, 10,644 bytes,
sha256 `6515dcdd…d530`). Decoding it confirms the supplied server's writer
(`Gateway.MapChat/Map.cs:15-32`): each record is a `true` flag byte, then 13 little-endian fields
(`Id, NameId, LocationId, X, Y, Z, Heading, IconId, NotificationType, SubNameId, Unknown11,
BreadcrumbQuestId, TeleportLocationId`), then a u32-length-prefixed ASCII `AtlasName`, then one
trailing int (`Unknown15`); a final `false` flag ends the list. The arithmetic closes exactly:
149 records × 61 bytes + 1,541 name bytes + one terminator = 10,631 body bytes, which is what the
frame carries. Every one of the 149 records matches
`working/OSFR.Server/OSFR Server/Customize/PointOfInterestDefinitions.json` field for field, so the
JSON table is exactly the payload this server sends for opcode 57; the original client is the
consumer of that message id in the recorded gateway milestone, but the map/POI panel itself was not
re-inspected here.

Captured table facts: 149 definitions; every id unique; notification types 0 (14 rows), 3 (43),
6 (74) and 7 (18); atlas names `sacredgrove` (135), `sunstone` (7) and empty (7); all 149 rows carry
a teleport location id and 43 carry a breadcrumb quest id. The pinned community project labels the
types `Dungeon=3`, `Local=6`, `ZoneHub=7`, `Ruins=10`; its serializer writes the same field order but
omits the per-record boolean, so it is a partial cross-check that the captured bytes complete.
Six name ids are shared by more than one definition (`0, 2061, 382885, 426528, 436321, 441416`), so a
display name does not address a single place.

The client's own locale resolves the names: the 8 zone-hub rows (ids 3–10) are `Snowhill`,
`Briarwood`, `Sanctuary`, `Greenwood Forest`, `Cobblestone Village`, `Shrouded Glade`, `Lakeshore`
and `Briarheart`, i.e. exactly the range the Lua handler reacts to. 102 of the 103 distinct POI name
ids resolve; only name id 0 (the 14 unnamed rows) has no text.

### Exploration collections, their entries and their rewards

The client ships `Resources/CollectionCategories.txt`; row `1^93636^9535` is category id 1 and the
locale resolves name id 93636 to **Exploration**. The supplied initialization payload
(`PacketSendSelfToClient.json`, accepted by the original client in the recorded gateway milestone)
carries 253 collection rows for one template profile, **48 of them in category 1**, with 257 entries
and 220 collected entries.

The recovered link between a collection entry and a place is the **name id**: six captured
exploration entries name a location that also appears in the POI table, and each of those name ids
addresses exactly one definition:

| Entry (locale text) | name id | POI id | POI position (X, Y, Z) |
| --- | --- | --- | --- |
| Treeleaf's Retreat | 5133 | 78 | −710.791, −49.829, 1754.531 |
| Briarheart Caverns | 5707 | 66 | −38.802, −0.004, 2900.154 |
| Rainbow Lake | 93710 | 127 | −720.069, −6.887, 789.279 |
| Cray Caves | 71694 | 84 | −45.176, −31.049, −1172.106 |
| Singing Crystal Mines | 4806 | 30 | 407.055, 47.112, 345.557 |
| Hewey's Escape | 102010 | 89 | −447.579, −0.676, 326.010 |

The remaining 251 exploration entries have no POI row at all, so their coordinates are **not
recovered** — the POI table is a 149-row subset, not the full exploration location set.

Reward bundles in the same payload give a concrete reward chain for the rare and elite sets, using
the pinned community item catalogue only as an id-to-name-id index and the client locale for text:

* Snowhill Rare Exploration → item 18040 (`Pathfinder's Pants`);
* Shrouded Glade Rare Exploration → items 15322 (`Seeker's Backpack`) and 17820 (`Seeker's Hat`);
* all eight elite sets → `Elite Explorer Backpack of <region>` (items 70341-70348) plus a token and
  `RewardBundleEntryCollectionAdd` pointing at collection 309, `Elite Explorer`;
* the common sets (for example Briarwood - Thistlerow, Sanctuary) carry a
  `RewardBundleEntryToken` (token ids 199, 211, …) instead of an item.

Across the 48 exploration rows the payload carries 31 token entries, 27 item entries, 8 cascading
collection entries and one quest entry. The wiki's reward text agrees: `Pathfinder Pants`,
`Seeker's Hat` and `Seeker's Backpack` appear on `Exploration (Collections)` (revision 46070), and
`Collections` (revision 45236) states that "Approaching any location in the list will activate the
corresponding collection" and that rare-exploration entries are sparkling coins that must be
clicked. No numeric experience value is recovered anywhere: the wiki only names "Adventurer Exp.",
and the token ids it presumably pays through are unrecovered names.

## What is reconstructed (this module's own policy, labelled)

| Rule | Value | Why |
| --- | --- | --- |
| Activation radius | caller argument, validated finite and ≥ 0 | the real radius is unrecovered; no number is invented |
| Trigger shape | point proximity, planar X/Z distance | the definition carries one position; the vertical axis is ignored |
| Candidate set | definitions matching an optional caller-supplied `location_id` filter | which zone maps to which location id is not modelled here |
| Selection | nearest in-range candidate wins | nearest-first is the only rule the data supports |
| Once-only | a discovery fires once per character; a replay returns `AlreadyDiscovered` | every captured entry carries a per-profile `collected` flag |
| Ambiguous names | a shared name id is not resolved to one POI | six name ids address several definitions |
| Rewards | not modelled; the module reports `Discovered{definition}` and nothing else | the lead-owned economy and item 074 own grants |

## Verification

| Check | Command | Result |
| --- | --- | --- |
| Rust RED | `.\scripts\cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml --test piece_073` | `couldn't find file tests\..\src\restoration\piece_073.rs` |
| Rust RED (first revision) | same | 2 failed: a fixture heading and the planar-distance expectation |
| Rust GREEN | same | 11 passed; 0 failed; 1 ignored |
| Private gate | env-pinned captured message + table, `--ignored --exact private_captured_poi_payload_matches_supplied_table` | 1 passed; 149 definitions decoded from the real payload and compared field for field |
| Neighbour regression | `--test piece_038` | 15 passed; 0 failed |
| Independent checker | `python scripts\pieces\piece_073.py --output …\piece-073-evidence.json` | exit 0; 149 definitions, 48 exploration collections, 6 linked entries, 5 vectors agreed |
| Python tests | `python tests\pieces\test_piece_073.py` | 16 passed; includes changed-capture-byte and changed-Lua-column rejection |
| Format | `rustfmt --check --edition 2024` on the two owned Rust files | clean |

Full transcripts and hashes: `lab/reports/piece-073-20261009T0449Z/` (`verification.txt`,
`source-provenance.txt`, `piece-073-evidence.json`, `locale-en_us.json`, `rust-vectors.json`).

## Explicit gaps

1. **Activation radius and trigger shape are unrecovered.** Nothing in the client data, the captured
   payload or the community projects carries a proximity radius; `Unknown11` (651 in most rows) and
   `Unknown15` (0-100) are unnamed and are not asserted to be one. Bounded search: the client's POI
   definition accessor (`pseudocode.c:1573569`) only maps data-source columns onto the definition
   struct, and the definition manager (`:2425374`-`:2426911`) only stores, loads and looks up
   definitions — no radius field or distance constant appears on either path.
2. **251 of 257 exploration entries have no POI row.** The 149-row table covers a fraction of the
   listed locations, so most exploration trigger coordinates are still missing evidence.
3. **Notification type 0 has no recovered name** (14 rows), and community type 10 (`Ruins`) does not
   appear in the supplied table at all.
4. **Token rewards are unresolved.** Token ids are captured per collection, but the id-to-meaning
   mapping (presumably the "Adventurer Exp." the wiki names) is not recovered.
5. **The POI table's provenance is the supplied emulator**, not a first-party data file; the wire
   equality proves the emulator sends exactly this table in exactly this shape, not that the
   emulator transcribed the original server table without loss or invention.
6. **No native acceptance.** Nobody walked to a POI in the client with this module in a server; the
   lead's native visit validation is still the gate that would make any of it playable.

## Next gates for the lead

1. Register `pub mod piece_073;` under `src/restoration/` and drop the `#[path]` include from
   `tests/piece_073.rs`.
2. Decide the server-side radius for a first playable slice, wire `DiscoveryTracker` into the
   position update path, and emit the existing collection update packets for entries whose name id
   resolves to the discovered POI — item 074 owns item collections and item 038 owns the collection
   reconciliation model.
3. Before any reward claim, extend the recovered table with the remaining exploration locations or
   keep the module's no-grant contract.

Cross-links: [[038 Adventurer progression]], [[../Design/MMO Worker Contract]],
[[../../MMO Work Queue]].
