# 029 Housing decoration discovery

Status: fixture-action gate implemented and locally verified, 2026-10-09 03:40 UTC. Evidence label:
**ORIGINAL client vocabulary and wire shape over a RECONSTRUCTED gate policy**. There is no native
housing acceptance claim: no client run, no lot, no decoration was placed, moved or removed.

Owned paths used:

| Path | Role |
| --- | --- |
| `prototype/osfr-tools/src/restoration/piece_029.rs` | action vocabulary, instance selection, gate |
| `prototype/osfr-tools/tests/piece_029.rs` | 16 owned vectors, `#[path]` include, private gate |
| `scripts/pieces/piece_029.py` | independent checker over the shipped client files |
| `tests/pieces/test_piece_029.py` | 16 stdlib tests, including the exported-vector cross-check |
| `lab/reports/piece-029-20261009T0340Z/` | RED/GREEN, private gate, checker, suite logs |

No shared `lib.rs`, manifest, gateway, launcher, content, save or release path was touched, and no
other item's module or interface was overwritten.

## What is recovered (ORIGINAL)

### Fixture actions are a shipped name vocabulary, not free text

`Resources/StringHashToValue.txt` (sha256
`22003f3224dcc1cd5a189e6e818ab8c82711e713183f04dba13b76c255aeb570`) carries exactly three
fixture-action names:

| Config key | Action name |
| --- | --- |
| `Housing.ElevateFixtureActionName` | `Elevate` |
| `Housing.MoveFixtureActionName` | `MoveFixture` |
| `Housing.RemoveFixtureActionName` | `RemoveFixture` |

The client asks the server whether one of these is permitted for a specific fixture:
`client-ui/ScriptsBase.lua` (sha256
`c7c71c7cbb177c975a7cdb12ed0c2e1b74d662f5ca5ef6a4e9ff86d2d332a8f6`) line 33801 calls
`House.IsActionPermitted(currentItemGuid, "RemoveFixture")` and line 33822 calls the same with
`"MoveFixture"`. Those are the only two names that reach `IsActionPermitted` anywhere in the
decompiled UI, so `Elevate` is shipped vocabulary without a recovered call site (gap listed below).

`FixtureDefinition` in the pinned community packet model
(`FreeRealms-Legacy/src/Sanctuary.Packet.Common/Housing/FixtureDefinition.cs`) carries exactly this
shape: `ItemDefinitionId`, `ModelId`, `Category`, `LuaCall`, an `Actions` map of
`FixtureAction { Id, Name, RequirementId }` and `CompositeEffectId`. The shipped client allocates the
matching container `SoeUtil::HashListMap<int, FixtureDefinition::FixtureAction, 16, -1>`
(`pseudocode.c:1402661`, also `:1402888`, `:1403295`, `:2461032`, `:2463156`) and keys fixtures twice
more by group and by type (`SoeUtil::HashMap<int, FixtureByGroupEntry, …>`,
`SoeUtil::HashMap<int, FixtureByTypeEntry, …>`, `pseudocode.c:1402798`, `:1402809`). The UI agrees:
`House.GetFixtureGroupByType("Floor"|"Wall"|"Roof")`, the `"Exterior"` group
(`ScriptsBase.lua:33244`, `:33265`), and `House.DoesGroupHaveCustomizableFixture(group, type)`.

### An interaction addresses one fixture instance by guid

* The radial menu reads its addressee from a single data source,
  `BaseClient.Housing.Instance.SelectedFixture` (`ScriptsBase.lua:33358`, update handler at
  `:33922`–`:33930`), and stores it as `currentItemGuid`. `-1` means nothing is selected and the
  menu is closed (`:33925`).
* Every fixture-scoped message is keyed by fixture guid: `HousingPacketFixtureRemove`,
  `ClientHousingPacketPickupFixture` (one `u64`), `ClientHousingPacketSaveFixture`, and the client's
  own debug line `Selected House Fixture: %lld, Name: %s, Loc: %.2f, %.2f, %.2f, H: %.2f`
  (`pseudocode.c:1397564`). The strong type `SoeUtilSignatureFixtureGuid` types those keys.
* An item definition id is *not* an instance identity: `FixtureInstanceInfo` pairs a fixture guid
  with an `ItemDefinitionId`, and `HousingPacketFixtureItemList` lists owned items next to placed
  fixtures. The wire also carries definition-only requests
  (`ClientHousingPacketPreviewByItemDefinitionIdRequest`, inner opcode 20, body one `u32`; see item
  025's note), which cannot name one of two identical decorations.

### A named fixture action is a tracked game event

The shipped client has an objective type for exactly this: `ObjectiveHousingFixtureMovePacket` is
built with objective id `0xc` (`pseudocode.c:1191058`–`:1191059`), the objective factory
`FUN_00cf3aa0` constructs `ClientObjectiveHousingFixtureMove` and stores a fixture id and an action
(`pseudocode.c:1840204`–`:1840206`), and the parser logs
`Adding HousingFixtureMove objective %d - fixture id %d, action %d` (`pseudocode.c:1702798`).
The neighbouring case `0xb` builds `ObjectiveFirstMovementPacket`, so the type numbering is
consistent. Performing a named action on a fixture is therefore client-observable state, not only an
edit-mode UI gesture.

### The interactive-furniture side of the client

The recovered `Fixture` actor (`Fixture::vftable`, `pseudocode.c:1795360`) carries an array of
character guids (occupants) and a per-character effect map
`HashListMap<StrongType<u64, SoeUtilSignatureCharacterGuid>, Fixture::EffectStorageNode>`
(`pseudocode.c:1795382`–`:1795396`), i.e. a fixture can hold per-character interactive state.
Fixture models are resolved through the asset system, and a missing one logs
`FixtureActorErrors.log` `Failed to find ModelDefinition for modelId=%d.` (`pseudocode.c:1409677`).
`Resources/StringHashToValue.txt` also ships `Housing.StaticSeatingDisabled^1`, the client switch for
its static seating path, and `RatingSystem.HousingId^Housing`, the rating-system key for house
ratings.

### Requirements a fixture action points at

`Resources/Requirements.txt` (sha256
`1dc2f97fd35fcc20b55213ba8f08e93c43c84f442c139f08c0c816d5f7012644`) is the shipped requirement
table: **43** rows (`client_requirements.ID` 1..43) over six
`ref_client_req_types.CODE_FACTORY_NAME` values — `CheckRankLastProfileByType`, `AlwaysFails`,
`SourceVipRank`, `CheckCurrentProfile`, `CheckRenamePending`, `CheckNudge`. A `FixtureAction`'s
`RequirementId` resolves against it. The companion tables `RequirementGroups.txt` (39 rows) and
`RequirementSets.txt` (34 rows) group and set those ids.

Adjacent decoration-placement tables exist but were only inventoried, not consumed:
`HousingPlacementData.txt` (`#ID^TYPE^ASSET_NAME`, two developer rows `1^1^test.adr`, `2^1^test2.adr`),
`HousingPlacementAssociations.txt` (one parent/child row) and `HousingNames.txt` (two string ids).

## What is reconstructed

All of the following is this module's own bounded policy, not recovered server code. The server-side
fixture-definition table — which definition declares which actions, its `LuaCall` text and its
`CompositeEffectId` — is **not** recovered, so definitions are caller input.

| Rule | Value | Why |
| --- | --- | --- |
| Edit permission | `can_edit` required for every gate evaluation | `HousingInstance_DS.CanEdit` gates the shipped edit UI |
| Instance selection | guid first; an item-definition address is accepted only when it names exactly one instance of that lot | two identical decorations share an item definition, and "first match" would act on the wrong instance |
| Ambiguity | refused (`AmbiguousSelection`), never guessed | same |
| Interaction ids | only 1..5 are valid | the five `Housing.INTERACTION_*` constants |
| Named gates | `PICK_UP` requires `RemoveFixture`; `MOVE` requires `MoveFixture`; farming additionally requires `MoveFixture` before removal | recovered Lua gates, applied strictly |
| `Elevate` | ungated | no recovered call site passes it to `IsActionPermitted` |
| Requirement 0 | no requirement declared | ids in the shipped table start at 1 |
| Other requirement ids | must exist in the requirements table, else refused | refusing is the safe reading of an unknown gate |
| Requirement evaluation | left to the caller; the plan only carries the id | the factory operator semantics are not recovered |
| Members-only fixture | a non-member may only remove it | `MakeHousingRadialMenu` offers only the remove button to a non-member |
| Effects | the plan is data only: nothing is placed, moved, deleted, minted or paid | lead-owned economy and item 006/026 own state |

The gate's output is one `InteractionPlan` (lot, fixture guid, definition id, item definition id,
interaction id, the recovered `House.*` call name, the most specific named gate and its requirement
id) or one typed `Refusal`. Refusal order is documented in the module and pinned by tests:
edit permission, instance selection, interaction id, definition presence, members-only policy,
declared action, known requirement.

## Verification

Commands run from `%USERPROFILE%\Documents\Projects\CartoonMMO\project`.

| Check | Command | Result | Log |
| --- | --- | --- | --- |
| Rust RED (module absent) | `.\scripts\cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml --test piece_029` | `error: couldn't find file tests\..\src\restoration\piece_029.rs` | `red-missing-module.log` |
| Rust RED (behaviour) | same, first module revision | 3 failed: ambiguous item definition accepted, `MoveFixture` gate skipped, unknown requirement allowed | `red-first-revision.log` |
| Rust GREEN | same | 16 passed, 0 failed, 1 ignored | `green-final.log` |
| Private gate | `OSFR_PIECE_029_CONFIG/…/_LUA/…/_REQUIREMENTS/…/_REPORT` + `-CargoArgs @('test',…,'--','--ignored','--exact','private_decoration_vocabulary_matches_shipped_client_files')` | 1 passed; re-derives the 3 names, 5 interactions, 43 requirement rows and 6 factories from the shipped files and writes the vectors | `private-gate.log` |
| Regression sweep | `--test housing --test piece_025 --test piece_026` | 26 + 8 + 12 passed, 0 failed | `neighbour-suite.log` |
| Library suite | `--lib` | 24 passed, 0 failed | `lib-suite.log` |
| Independent checker | `& '..\lab\tools\wiki-env\Scripts\python.exe' scripts\pieces\piece_029.py --output <report>` | exit 0; 22 vectors, 22 agreements, 0 disagreements, 0 vocabulary mismatches | `independent-checker.log` |
| Python tests | `& '..\lab\tools\wiki-env\Scripts\python.exe' -m unittest discover -s tests\pieces -p "test_piece_029.py" -v` | 16 passed, 0 failed | `python-tests.log` |
| Format | `rustfmt --check --edition 2024` on the two owned Rust files | clean | `rustfmt-check.log` |

The RED cycle changed the implementation twice, not the tests: `Selection::ItemDefinition` now
refuses an ambiguous instance instead of taking the first match, and the housing `MOVE` gate now
requires the declared `MoveFixture` action. A third defect was found by the private gate rather than
by a unit test: the Lua interaction constants are table rows (`INTERACTION_PICK_UP = 1,`) with no
`Housing.` prefix, so the first parser revision extracted nothing; the checker then caught a mirrored
prefix bug in its own parser (it re-prefixed an already-prefixed name).

The whole-crate `cargo test` still does not complete, and the failure is in other workers' in-flight
files: `tests/piece_038.rs` has no module and `tests/piece_035.rs` has an ambiguous numeric type. The
two files this item owns compile and pass on their own. Full log: `full-suite-attempt.log`.

The independent checker is a second implementation, in another language, of the same gate. It
re-derives the vocabulary from the shipped files, then re-evaluates the 22 vectors the Rust private
gate exported (including guid resolution, ambiguity, cross-lot addressing, a foreign guid, a
missing definition, members-only guests, farming gates, a known requirement, an absent requirement
and a read-only lot) and compares field for field.

## Deliberate gaps and remaining gates

1. **Lead integration.** The module is a `#[path]` include; register it in `src/lib.rs` and drop the
   include in `tests/piece_029.rs`. Nothing else here needs to move.
2. **The server-side fixture-definition table is unrecovered.** Which definition declares which
   actions, the `LuaCall` text and the `CompositeEffectId` all live on the original server. Until
   that data is recovered, definitions are caller input and no original-content claim can be made
   about a specific decoration.
3. **`Elevate` has no recovered call site.** The name is shipped in the client config, but nothing in
   the decompiled UI passes it to `IsActionPermitted`, so vertical movement stays ungated here and is
   listed as an open question rather than being wired to the up button by analogy.
4. **Requirement semantics are unrecovered.** The gate checks that an id exists; the operator column
   and the six `CODE_FACTORY_NAME` evaluators are client-side code whose server behaviour is unknown,
   so evaluation stays with the caller.
5. **Wire.** The client requests that would carry an interaction
   (`ClientHousingPacketPlaceFixtureRequest`/`PickupFixture`/`SaveFixture`) are decoded by item 025,
   and the server responses are still undecoded. This gate is exercised by its own vectors only.
6. **State.** Placement, move, pick-up, tint and persistence belong to item 006 (module
   `housing.rs`), conservation to item 026 and snap/rotation/build bounds to item 027. This module
   produces plans for those layers and never mutates their state.
7. **Per-character fixture effects and seating are not simulated.** The client's occupant array and
   `Fixture::EffectStorageNode` map are recorded as recovered structure; their message flow is not
   recovered, and `Housing.StaticSeatingDisabled^1` is reported as a config fact only.
8. **Native acceptance (item 030).** Entering a lot, selecting a decoration and using the radial menu
   against a real client is still the only gate that proves the reconstructed server answers are
   accepted.

Cross-links: [[Housing Restoration]], [[024 Furniture catalogue]], [[025 Furniture placement client
wire]], [[026 Housing inventory transactions]], [[../../MMO Work Queue]], [[../../Design/MMO Worker
Contract]].
