# Housing Restoration

Status: scoped module implemented, then independently reviewed and repaired, 2026-10-08 19:50
Pacific. This is a bounded state machine, not a native housing acceptance claim. Its evidence label is
**RECONSTRUCTED server behaviour over ORIGINAL packet and property facts**.

Owned paths: `prototype/osfr-tools/src/housing.rs`, `prototype/osfr-tools/tests/housing.rs`,
this note, `notes/Reviews/Housing Independent Findings.md`, and the private evidence directories
`lab/reports/housing-restoration-20261008T191409Z/` and
`lab/reports/housing-independent-20261009T0243Z/`. No shared file, manifest, gateway, content, save,
launcher or release path was modified.

## What was recovered

Package identity is ORIGINAL. Housing is a packet family of base opcode **127**
(`BasePackets.BaseHousingPacket`), and every member number below appears identically in the supplied
client and in the decompiled community server model.

| Inner opcode | Packet | Direction |
| --- | --- | --- |
| 1 | `ClientHousingPacketPlaceFixtureRequest` | client to server |
| 2 | `ClientHousingPacketPlaceFixture` | server response |
| 3 | `ClientHousingPacketPickupFixture` | client to server |
| 5 | `ClientHousingPacketSaveFixture` | client to server |
| 6 | `ClientHousingPacketSetEditMode` | client to server |
| 28 | `HousingPacketInstanceData` | server to client |
| 39 | `HousingPacketInstanceList` | server to client |
| 40 | `HousingPacketFixtureUpdate` | server to client |
| 41 | `HousingPacketFixtureRemove` | server to client |
| 53 | `HousingPacketFixturePlacementDenied` | server to client |

Client construction was confirmed in the Ghidra export rather than taken from the table alone:
`FUN_00aa5340` builds `ClientHousingPacketPlaceFixtureRequest` after `FUN_00aa4990(1)`,
`FUN_00aa53f0` builds `ClientHousingPacketPlaceFixture` after `FUN_00aa4990(2)`, and `FUN_00aa81a0`
builds `ClientHousingPacketPickupFixture` after `FUN_00aa4990(3)`. The same module finds
`ClientHousingPacketSaveFixture`, `ClientHousingPacketPayUpkeep`, `HousingPacketFixtureRemove`,
`HousingPacketUpdateFixturePosition` and `HousingPacketZoneData`, so the family is not a single
packet.

Two further ORIGINAL client strings bound placement: `HousingMaxFixtures` is read from the property
definition before a placement is answered, and a failed check prints `Fixture placement denied.`
No numeric limit is attached to that string in the recovered code.

A placed fixture (`FixtureInstance`) carries a fixture guid, its house guid, a fixture definition id,
position, rotation, a tilt quaternion, an npc guid, a tint id, customization data, an optional XML
blob and a scale. A housing instance (`PlayerHousingInstanceData`) carries the house guid, the owner
guid and name, a name id and house name, maximum fixture and landmark counts, the fixture map, a
permission map, current fixture and landmark counts, build areas, an icon, a furniture score, a
members-only flag and a factory instance. `InstancePermission` names level **3** as "can edit".

Property rows in `FreeRealms-Legacy/src/Resources/Houses.json`
(sha256 `854b2d3f009b916a928f66d57a42e2bf685dcc853e898d149580eefd26196d3c`) are ORIGINAL community
content: 31 rows with a house id, name id, zone id, icon, spawn position, spawn rotation and one or
more `BuildAreas` min/max pairs. Row 1 is used verbatim as the test fixture: `Id` 1, `ZoneId` 1,
`NameId` 6514, and a build area of x 322.0 to 443.3, y -100.0 to 60.0, z 392.1 to 477.8.

Ownership gating was recovered from the pinned community server: `ZoneManager.PlayerOwnsHouse(ownerId,
zoneDefinitionId)` resolves a housing zone to its owning character, and housing zones additionally
require a non-null owner. The read-only wiki corpus agrees at a design level: [Lots](https://freerealms.fandom.com/wiki/Lots),
revision 49783, describes lots as housing areas the player owns and builds on, with one free
apartment for everyone and additional houses as claimed or purchased upgrades. The wiki also records
furniture as both indoor and outdoor items, some interactive and most plain furniture.

## What the module reconstructs

Everything below is invented server behaviour chosen to be defensible, bounded and reversible. None of
it is a recovered Free Realms number or rule.

- The fixture guid mint: a hashed function of lot guid, fixture definition id and captured instance
  id, in a reserved `0x0f00_0000_0000_0000` block. Real fixture guids are server-assigned and unknown.
- The lot guid derivation: an FNV-1a hash of owner and property id, reserved to `HOUSE_GUID_BASE` so a
  restored lot can never collide with a live gameplay zone guid.
- Ownership transitions: one grant per property. A second grant of the same property to a different
  owner is refused rather than silently reassigned.
- Furniture conservation: a `FixtureItemLedger` counts owned and placed units per definition and
  tint, and tracks which captured instance ids may be placed. Placing stages a `Consume`, picking up
  stages a `Return`, and moving rekeys the tint only. A rejected request leaves both ledgers at their
  previous values because every mutation runs on a clone and commits only after validation.
- Placement limits: `property.max_fixtures` is the only bound, because that is the only limit the
  recovered client names. Landmark limits are carried but never enforced; no landmark semantics were
  recovered.
- Intrinsic bounds: finite components, `|component| <= 1_000_000.0`, positive scale, and a unit
  quaternion within `1e-3` for rotation and tilt. These are local defensive bounds in the same spirit
  as `character_store::MAX_COMPONENT_MAGNITUDE`, not recovered world or angle limits.
- Movement while a lot is locked is refused, and a stranger without a level-3 permission is refused
  with `NotPropertyOwner`. Failure ordering is: lot lookup, permission, lock, request validity, build
  area, duplicate instance, inventory, commit.

## Deliberate gaps

These are not implemented and must not be implied:

- no collision, snap, drag preview, floor or surface detection, or out-of-bounds physics. The build
  area test is a box test on the recovered property rectangle, which is a proxy, not native collision;
- no landmark scoring, furniture score arithmetic, upkeep billing, house rating or voting;
- no visitor access, invitations, neighbours, house naming or house locking semantics beyond a local
  boolean;
- no fixture customization, composite effects, `LuaCall` behaviour or interactive furniture;
- no packet writer for the housing family, so nothing in this module has been accepted by the native
  client;
- `HousingPacketFixtureAsset`, `HousingPacketFixtureItemList` and `HousingPacketZoneData` payload
  layouts outside the community model are undecoded. The community gateway implements only
  `ClientHousingPacketSetEditMode` and `ClientHousingPacketEnterRequest`, so placement, pick-up and
  save have **no** community server reference implementation to cross-check;
- the recovered `HousingPacketFixturePlacementDenied` body is unknown, so a denial reply cannot be
  serialized yet.

## Verification

`prototype/osfr-tools/tests/housing.rs` includes the module by `#[path]` so it compiles before the lead
registers `pub mod housing;` in `src/lib.rs`. The suite of **26** passes; see
[[Housing Independent Findings]] for the review that raised it from 16. The original candidate's
per-target sweep is `lab/reports/housing-restoration-20261008T191409Z/regression-full-suite.txt`; that
directory holds only that log, so this note no longer claims RED-first text or a sweep receipt from
it. The review's own RED, GREEN, sweep and source-verification logs are in
`lab/reports/housing-independent-20261009T0243Z/`. Three vectors changed the implementation rather
than the test: a tint/definition pair that minted two different guids for one identity, a
definition-blind duplicate check that rejected a distinct table sharing a captured instance id, and a
snapshot that restored limits and build areas without persisting them.

The independent pass found and repaired eight demonstrated defects — six with a RED test before the
fix, plus two dead or misleading functions removed: a rejected snapshot load adopted the rejected
document's content table into live state; a snapshot with two rows for one lot guid silently dropped
the first lot and its fixtures; replacing a lot's cap or build area left placement reading the stale
property table, so an over-cap placement was accepted and then could not be saved; the save path
trusted a client-supplied house guid and empty instance ids; a save that rewrote an existing fixture
skipped the build-area bound; a rejected `seed` partially consumed captured instance ids; and
`grant_property_or_default` (which never granted) with the private `mutable_lot` were removed. The
fixture cap `400` in the test fixture is a local value, not recovered content, and the test that used
it was renamed accordingly.

Two checks found real defects during the red-green cycle and are retained as evidence:

1. a chair and a table sharing an item instance id minted the same fixture guid, and the second
   insertion silently overwrote the first. The mint now includes the fixture definition, insertion
   refuses an existing guid, and the duplicate check is definition-aware;
2. a restored lot carried no build areas or fixture limits, so a restart could not validate either.
   `LotSnapshot` now carries `count` and `build_areas`, and `load_state` rebuilds the property table
   from the restored lots.

## Next gates

1. Lead integration: register `pub mod housing;` in `src/lib.rs` and drop the `#[path]` include.
2. Wire integration (item 025): decode or capture `PlaceFixtureRequest`, `SaveFixture`,
   `PickupFixture` and `HousingPacketFixturePlacementDenied` bodies, then answer from this state
   machine. Item 025 depends on this module's interfaces.
3. Inventory integration (item 026): map `FixtureItemLedger` proposals onto the lead-owned inventory
   transaction so a placement and its stack change commit or roll back together. After a restart the
   owning inventory reseeds the captured instance ids and then re-stages the lot's persisted fixtures
   into the ledger; the verified recipe is `restart_conservation_requires_restaging_the_captured_instances`
   in `tests/housing.rs`, and a pick-up with no staged instance is still refused.
4. Property acquisition (item 021): recover purchase and grant semantics, currency accounting and
   once-only ownership from original data before replacing the single local grant.
5. Native acceptance: enter a housing lot from the native client, place, move, pick up and re-enter
   after a server restart. Only that proves the client accepts the reconstructed rules.
6. Save migration (item 028): versioned restore of older snapshots once real saves exist. The current
   schema rejects an unknown `schema_version` and requires an exact `count`.

Cross-links: [[Gameplay Coverage]], [[Universal Modder Assessment]], [[../Design/MMO Worker Contract]],
[[Wiki Research Database]].
