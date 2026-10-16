# Housing service integration

Status: implementation + focused tests complete, 2026-10-09 (UTC). Owned paths:
`prototype/osfr-tools/src/housing_service.rs` (new), `prototype/osfr-tools/src/housing.rs`
(four necessary fixes), `prototype/osfr-tools/tests/housing_service.rs` (new), this note and
`lab/reports/housing-service-repair-20261009/`. No shared path, content, save, launcher, release or
UI file was touched. Skills: RTK (normal commands), Ponytail (one service, no second ledger,
no new dependency), Caveman (terse reporting, complete persisted notes), reverse-engineering
(client writer/reader anchors), game-research (pinned community sources first; no new web work).

**No native acceptance is claimed.** No housing packet has been exchanged with the shipped client
through this service. The gateway hook below is the lead's integration step.

## What was built

`housing_service.rs` is the callable seam between the recovered native housing requests, the
restored `housing` state machine and the live character `Inventory`. It answers the review finding
that housing "cannot see or keep anything": it routes the recovered client family, consumes the
real backpack, and returns one serde state object for the canonical snapshot.

| Item | Shape |
| --- | --- |
| `FurnitureStock` | `instances()`, `consume(instance_id, qty)`, `return_instance(definition_id, tint_id)`. `impl FurnitureStock for Inventory` is the live path (`state()`, `consume`, `grant_with_tint`); tests use a small double so they never need private content |
| `HousingService` | `new(config)`, `from_state(config, &state, &inventory)`, `state()`, `lot(house_guid)`, `grant_property`, `register_property`, `set_locked`, `set_permission`, `offer(actor, selection_id, instance_id, &stock)`, `handle(actor, inner, &mut stock)`, `move_fixture(actor, house, fixture, transform, &mut stock)` |
| `Request` / `decode_request` | the recovered client bodies: 2 place (`WireTransform`), 3 pickup, 6 edit mode, 19 enter, 1/5/20 as typed non-actions |
| `ServiceOutcome` | `replies: Vec<HousingReply>`, `changed: bool` (the lead persists `state()` when `changed`) |
| `HousingReply` | `FixtureUpdate`(40), `FixtureRemove`(41), `HouseInfo`(44), `PlacementDenied`(53), `Unrecovered { opcode }` for 28/39/42/51; `opcode()`, `encode()`, `encode_inner()` |
| `ServiceError` | typed refusal + `denial_reply()`/`denial_message()` for the recovered 53 body |
| `HousingServiceState` | `{ schema_version: u32, lots: Vec<housing::LotSnapshot> }`, `Clone + Serialize + Deserialize + deny_unknown_fields` |
| `FixtureConfig` | reconstructed `fixture_definition_id -> item_definition_id -> model_id` rows plus per-row provenance; `capture_defaults()` carries the seven captured triples |

## Evidence base

| Input | Use |
| --- | --- |
| `lab/references/OpenSourceFreeRealms/ofrserver/Packets/p2.pcap` (`65ac0801...`) | captured flow `2 -> 40 + 44`, `3 -> 41`, `5 -> 51`, `1 -> 42`; the reply pairing and the `127/40` fixture/item/model triples |
| `lab/decompiled/client/FreeRealms/pseudocode.c` (`f84a5867...`) | request writers `FUN_00abcb90` (1), `FUN_00ac0510` (3), `FUN_00abcb30` (6), `FUN_00abca80` (19), `FUN_00abcd90` (20); the `127/2` placement object `+0x40`; the 53 string reader `FUN_00ab3030` |
| `lab/references/FreeRealms-Legacy/src/Resources/Houses.json` | property rows with build areas (test vector shape) |
| `notes/Research/Housing Placement Selection Identity.md` | the `127/2` first dword is a placement selection id, not an item definition and not an inventory identity |
| `notes/Research/Housing Native Placement Codec.md`, `Housing Placement Responses.md`, `Housing Fixture Tail Compatibility.md` | the 40/41/44/53 bodies and the 133 + 94 layout this service reuses through `housing_native` |
| `src/restoration/piece_025.rs` | the recovered request-body transcription, reused through `#[path]` because the restoration modules are not registered yet |

## Original versus reconstructed

ORIGINAL (routed, not invented): the family opcode and inner opcodes; the request body layouts;
the `127/2` selection-id field; the `127/40` fixture record / definition block / 94-byte tail
layout; the `127/41` single guid; the fixed `127/44` record with its captured constant fields
(`unused` = 10, `is_flora_allowed` = 1); the `127/53` single string; `Houses.json` build areas.

RECONSTRUCTED (must be replaced by recovered content/behaviour before any original claim): the
fixture configuration table; the selection-id binding; the mapping of the wire transform to
`housing::FixtureTransform`; the reply pairing after a pickup; every error string; the 40 tail's
positional fields (empty list, three zero words) and the definition flags/category/lua/actions;
the fixture guid minting and the numeric bounds, all inherited from `housing.rs`.

## Codec-required unresolved fields

These stay typed gaps, never invented values:

1. **`127/2` selection id.** The first dword is bound to a real instance only through
   `HousingService::offer`. A dword that happens to equal an inventory instance id places nothing
   (test `a_selection_id_is_never_an_inventory_identity`). The `offer` binding itself is the
   reconstruction allowed by the captured `127/1 -> 127/42` echo; the catalogue that owns the
   selection id is still unrecovered.
2. **`127/5` `SaveFixture`.** `quad_a`/`quad_b` are four `u32` words each, the `FUN_00abd1d0`
   block is opaque and nothing identifies the fixture, so `handle` answers `SaveFixture` with
   `ServiceError::UnresolvedRequest { opcode: 5, missing: [...] }`. Movement is reachable only
   through the typed `move_fixture` command until 5/51 are decoded.
3. **`127/51`, `127/42`, `127/28`, `127/39`.** Reply bodies are unrecovered, so the replies are
   `HousingReply::Unrecovered { opcode }`; `encode_inner()` on those returns
   `io::ErrorKind::Unsupported` instead of bytes. A pickup answers the recovered 41 plus an
   accurate 44; a placement answers the captured 40 + 44 pair.
4. **`127/40` 94-byte tail.** The service emits the codec's empty-list/zero-word shape, which is
   the captured 229-byte subtype body. Non-empty positional content is unrecovered.
5. **Tilt.** The `127/2` body carries no tilt, so a placement starts from the identity quaternion;
   the capture cannot contradict this.

## Invariants the tests pin

* one owner per lot: a stranger is refused at enter and at place; a guid with
  `PERMISSION_CAN_EDIT` may build but not reassign (tests `a_stranger_...`,
  `a_guest_with_edit_permission_...`);
* the live inventory is the only item owner: placement consumes the real instance (the backpack
  loses it), pickup returns one unit (definition and tint conserved, id re-minted by the
  inventory), and a refusal anywhere leaves both sides untouched;
* staged rollback: the lot is validated on a clone and committed only after the inventory
  mutation succeeded; a stock refusal, an out-of-area transform, a non-unit rotation, an unknown
  or ambiguous item definition, a stacked instance and a missing instance all leave
  `state().lots[0].fixtures.len() == 0` and the stock unchanged;
* restore is whole-document: schema version, `MAX_STATE_LOTS`/`MAX_STATE_FIXTURES`, the housing
  validator, cross-lot instance duplication and the inventory cross-check (`a fixture whose
  instance is still in the backpack is rejected`) all run before any state is adopted.

## Exact lead hooks

1. **Gateway dispatch (`src/gateway.rs` -> `Gameplay::handle_in_world`/`handle`).** Add a branch
   before the gameplay fall-through: when `inner[..2] == 127u16.to_le_bytes()`, call
   `HousingService::handle(guid, inner, &mut self.inventory)` (`guid` is the character guid the
   handler already receives). Wrap each reply with `encode_inner()`; a `Err(io::ErrorKind::
   Unsupported)` reply is an unrecovered body and must not be sent. Map `ServiceError` to a
   denial with `error.denial_reply()` only for a placement/pickup/edit request the client is
   waiting on; `MalformedRequest` and `UnresolvedRequest` should stay silent drops, matching the
   current swallowed behaviour but visible in the log.
2. **State (`src/gameplay.rs`).** Add the optional field exactly as the other candidates do:
   `#[serde(default, skip_serializing_if = "Option::is_none")] pub housing: Option<HousingServiceState>`
   to `GameplayState`, fill it from `HousingService::state()` when a housing request changed
   anything, and restore with
   `HousingService::from_state(FixtureConfig::capture_defaults(), state, &self.inventory.state())`.
   The existing atomic `game_store` write then persists housing next to `InventoryState`; the
   service never calls `housing::save_state`.
3. **Content.** `FixtureConfig` is serde; when the retail fixture table is recovered, load it from
   the package `content/` directory (e.g. `housing.json`) and pass it to `new`/`from_state`.
   `capture_defaults()` stays the explicit fallback with per-row provenance.
4. **Registration.** `pub mod housing_service;` in `src/lib.rs`; then drop the `#[path]` include
   from `tests/housing_service.rs` and switch the internal `piece_025` include to
   `crate::restoration::piece_025` once the restoration modules are registered.

## Checks and limits

Run with `scripts/cargo.ps1`:

* `--test housing_service`: 19 passed, 0 failed (`green.log`); the RED chain is
  `red-compile.log` (module missing) and `red-behavior.log` (10 failures: no inventory linkage, no
  rollback, ledger keyed on the wrong definition id);
* `--test housing --test housing_native --test housing_service --test piece_022 --test piece_025
  --test piece_026 --test piece_027`: 103 passed, 1 ignored (the private codec corpus gate), 0
  failed (`focused-neighbors.log`).

Limits: a furniture stack of more than one unit cannot back a placed fixture (the fixture identity
is the inventory instance id, and the capture has one instance per placement); a returned unit gets
a fresh instance id because `Inventory` re-mints on grant, so conservation is per definition and
tint, not per id; the placement ledger is rebuilt per request from the inventory plus the lot, so
two live characters editing one lot would need the lead's serialization; house-info counts,
furniture score, landmarks, collision and visitor permissions stay outside this scope.

Cross-links: [[Housing Restoration]], [[Housing Placement Responses]],
[[Housing Placement Selection Identity]], [[../Reviews/Delegated Housing Readiness]],
[[Pieces/025 Furniture placement client wire]], [[Pieces/026 Housing inventory transactions]],
[[../WorkItems/006]].
