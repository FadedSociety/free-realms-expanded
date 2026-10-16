# System: game.housing

93 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (30)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00aa5ec0` | `HousePlayerEventHandler::MatchesValue8` | HousePlayerEventHandler | Ordinary slot-2 predicate virtual on HousePlayerEventHandler. | [[Native Map/Shards/S0314 00aa2560-00aa90c0|S0314]] |
| `00aa5ed0` | `HouseActorEventHandler::MatchesValue8Or1` | HouseActorEventHandler | Ordinary slot-2 predicate virtual on HouseActorEventHandler. | [[Native Map/Shards/S0314 00aa2560-00aa90c0|S0314]] |
| `00aa5ef0` | `HouseTileEventHandler::MatchesValue8Or1` | HouseTileEventHandler | Ordinary slot-2 predicate virtual on HouseTileEventHandler. | [[Native Map/Shards/S0314 00aa2560-00aa90c0|S0314]] |
| `00aa5f30` | `NotifyNotInHousingEditMode` |  | Checks the housing edit-mode byte and sends a local warning when the action is unavailable. | [[Native Map/Shards/S0314 00aa2560-00aa90c0|S0314]] |
| `00aaac60` | `HousePlayerEventHandler::ctor` | HousePlayerEventHandler | Creates the event handler object that the housing editor registers for player-target events. | [[Native Map/Shards/S0315 00aa90e0-00aae6b0|S0315]] |
| `00aaad10` | `HousePlayerEventHandler::ScalarDeletingDtor` | HousePlayerEventHandler | Destruction path of the housing player-target event handler. | [[Native Map/Shards/S0315 00aa90e0-00aae6b0|S0315]] |
| `00aaad70` | `HouseTileEventHandler::ctor` | HouseTileEventHandler | Creates the event handler object that the housing editor registers for tile-target events. | [[Native Map/Shards/S0315 00aa90e0-00aae6b0|S0315]] |
| `00aaae20` | `HouseTileEventHandler::ScalarDeletingDtor` | HouseTileEventHandler | Destruction path of the housing tile-target event handler. | [[Native Map/Shards/S0315 00aa90e0-00aae6b0|S0315]] |
| `00aab720` | `SnapPositionToGridXZ` |  | Engine-side housing snap: the piece-027 note records this address as the function that snaps only x and z after House.SetGridSpacingInCentimeters stored 100/centimetres in the manager field +0x4188. | [[Native Map/Shards/S0315 00aa90e0-00aae6b0|S0315]] |
| `00aae3e0` | `HouseActorEventHandler::ctor` | HouseActorEventHandler | Third placement event handler of the housing editor (player/tile/actor), registered by the same housing code 00ab0a80 that builds the other two. | [[Native Map/Shards/S0315 00aa90e0-00aae6b0|S0315]] |
| `00aae440` | `TargetProcessorEventHandler::ctor` | TargetProcessorEventHandler | Third emission of the target-processor base construction stub, used by the actor event handler. | [[Native Map/Shards/S0315 00aa90e0-00aae6b0|S0315]] |
| `00aae490` | `HouseActorEventHandler::ScalarDeletingDtor` | HouseActorEventHandler | Destruction path of the housing actor-target event handler. | [[Native Map/Shards/S0315 00aa90e0-00aae6b0|S0315]] |
| `00aaf090` | `ApplyUseFreeFlyCameraOption` |  | Applies the housing editor's free-fly camera preference. | [[Native Map/Shards/S0316 00aae6e0-00ab2c20|S0316]] |
| `00aaf1b0` | `ApplyUseAutoOrientationOption` |  | Applies the housing editor's automatic-orientation preference. | [[Native Map/Shards/S0316 00aae6e0-00ab2c20|S0316]] |
| `00ab0d60` | `PlacementSelectionMode1Callback` |  | Handles one placement-selection mode callback; the selection identity is the paired lookup key described in the cited note. | [[Native Map/Shards/S0316 00aae6e0-00ab2c20|S0316]] |
| `00ab0f20` | `FilterPlacementConvexHits` |  | Called by the placement-transform routine to filter contact candidates; the housing purpose is based on caller 00ab1380. | [[Native Map/Shards/S0316 00aae6e0-00ab2c20|S0316]] |
| `00ab2250` | `PlacementSelectionMode2Callback` |  | Handles the second placement-selection mode; the selection identity is the paired lookup key described in the cited note. | [[Native Map/Shards/S0316 00aae6e0-00ab2c20|S0316]] |
| `00ab4690` | `ExportHouseFixtures` |  | Exports house fixture layout to a text file. | [[Native Map/Shards/S0317 00ab2c80-00ab7380|S0317]] |
| `00ab5af0` | `IsHousingActionPermitted` |  | Checks actions including moving, elevating and removing a fixture. | [[Native Map/Shards/S0317 00ab2c80-00ab7380|S0317]] |
| `00ab7380` | `HousePlayerEventHandler::OnSelectedFixture` | HousePlayerEventHandler | Forwards a player event to house fixture selection. | [[Native Map/Shards/S0317 00ab2c80-00ab7380|S0317]] |
| `00ac9180` | `HandleHouseSelectionEvent` |  | Selection/placement event handler of the house actor+formation tool; HouseActorEventHandler and HouseTileEventHandler forward their events here, and the admin/debug console gets a dump of the selected house actor. | [[Native Map/Shards/S0321 00ac8340-00acd3f0|S0321]] |
| `00ac9b30` | `HouseActorEventHandler::OnEvent` | HouseActorEventHandler | Adapter between the target-handler event stream and the housing selection/placement handler; the actor variant forwards the actor pointer, the tile variant (00ac9b80) forwards null. | [[Native Map/Shards/S0321 00ac8340-00acd3f0|S0321]] |
| `00ac9b80` | `HouseTileEventHandler::OnEvent` | HouseTileEventHandler | Tile-side adapter of the housing event stream: unlike the actor variant it has no actor to report, so the selection handler always takes its null-actor path. | [[Native Map/Shards/S0321 00ac8340-00acd3f0|S0321]] |
| `00cb9da0` | `HousePiece::DeletingDtor` | HousePiece | Scalar deleting destructor of HousePiece (slot 6 of the primary vftable). | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cb9dc0` | `HousePiece::Ctor` | HousePiece | Constructor of HousePiece: installs its primary and secondary vftables, its name string and parent link, then runs the lazy instance setup 00cb9b00. | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cb9f80` | `HousePiece::CtorWithStrings` | HousePiece | Second HousePiece constructor: takes name strings and a parent object, builds the parameter block for the piece's actor and creates its child objects when the flag is set. | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cba340` | `Fixture::Ctor` | Fixture | Constructor of Fixture, a HousePiece-derived housing object with IString members, a GUID array and a 32-bucket hash list map of EffectStorageNode values. | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cba4e0` | `Fixture::CtorFromObject` | Fixture | Second Fixture constructor: builds the same members as Fixture::Ctor and then fills placement fields from the object returned by the slot-4 lookup. | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cba720` | `Fixture::DtorBody` | Fixture | Destructor body of Fixture: releases its child object, hash list, GUID array and string members, then runs the HousePiece destructor body. | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cbb040` | `Fixture::DeletingDtor` | Fixture | Scalar deleting destructor of Fixture (primary vftable slot 6); the secondary adjustor 00cba4d0 forwards here. | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |

## 🟢 likely (34)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0097c680` | `HouseNameTemplateExpand` |  | Expands the HouseName template for a housing record, using the house_full_name key. | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `00aaacc0` | `TargetProcessorEventHandler::ctor ⚠downgrade` | TargetProcessorEventHandler | Base-class construction stub for the target-processor event handlers (HousePlayerEventHandler/HouseTileEventHandler/HouseActorEventHandler all reset themselves to this vtable in their scalar deleting destructors). | [[Native Map/Shards/S0315 00aa90e0-00aae6b0|S0315]] |
| `00aaadd0` | `TargetProcessorEventHandler::ctor ⚠downgrade` | TargetProcessorEventHandler | Second emission of the target-processor base construction stub, used by the tile event handler family. | [[Native Map/Shards/S0315 00aa90e0-00aae6b0|S0315]] |
| `00aaae80` | `CreatePreviewActorAtOffset` |  | Spawns the housing placement preview actor when the editor is in its preview mode byte at this+0x16; the created object lives at this+0x40bc and is consumed by the same cluster (00aaafc0, 00aab230, 00aab3b0, 00aab5c0). | [[Native Map/Shards/S0315 00aa90e0-00aae6b0|S0315]] |
| `00aaafc0` | `CommitPreviewTransformAndClear` |  | Applies the preview transform to the real actor and runs the settle/cleanup path of the housing editor; 00aab230 calls it to tear the current preview down before creating a new one. | [[Native Map/Shards/S0315 00aa90e0-00aae6b0|S0315]] |
| `00aab1a0` | `LookupHashedStringValue ⚠downgrade` |  | Name-keyed value lookup on the housing manager object (this+0x28b0 table); the caller is the script-facing binding 00c0b860, so this is the engine side of a House.* script call. | [[Native Map/Shards/S0315 00aa90e0-00aae6b0|S0315]] |
| `00aab230` | `RecreatePreviewActorFromTarget` |  | Rebuilds the housing placement preview from the currently targeted actor; called from the shared transform path 00ab1e30 and the editor entry points 00ab0d60/00ab2250. | [[Native Map/Shards/S0315 00aa90e0-00aae6b0|S0315]] |
| `00aab3b0` | `OrientPreviewToSurfaceNormal` |  | Auto-orientation of the fixture preview towards the surface normal, driven by the editor's rotation toggle; caller 00abb630 is the edit-mode update path. | [[Native Map/Shards/S0315 00aa90e0-00aae6b0|S0315]] |
| `00aab5c0` | `GetPlacementTransformMatrix` |  | Supplies the placement transform for the housing editor's selected or previewed fixture. | [[Native Map/Shards/S0315 00aa90e0-00aae6b0|S0315]] |
| `00aab6e0` | `SetSelectionId` |  | Selection setter for the housing editor: records the chosen fixture id, notifies the processor object and resets the cursor state when the selection is cleared. | [[Native Map/Shards/S0315 00aa90e0-00aae6b0|S0315]] |
| `00aab960` | `LookupNodeFlag74ByTargetId` |  | Checks a per-object flag in the housing processor's registry table for the currently targeted object. | [[Native Map/Shards/S0315 00aa90e0-00aae6b0|S0315]] |
| `00aae6e0` | `UpdatePreviewTransformAgainstSurface ⚠downgrade` |  | Used by the per-frame held-preview update; the housing preview role is based on caller 00abb630. | [[Native Map/Shards/S0316 00aae6e0-00ab2c20|S0316]] |
| `00ab41a0` | `LoadHousingNamesFile` |  | Loads local housing-name resources. | [[Native Map/Shards/S0317 00ab2c80-00ab7380|S0317]] |
| `00ab50d0` | `PrintFixtureTableRows` |  | Displays housing fixture table rows for debugging. | [[Native Map/Shards/S0317 00ab2c80-00ab7380|S0317]] |
| `00ab52d0` | `PrintFixtureTableCells → LogFixtureTableCellValues` |  | Performs the container or string operation identified by its body. | [[Native Map/Shards/S0317 00ab2c80-00ab7380|S0317]] |
| `00ab54b0` | `HandleHousingServerUnavailable ⚠downgrade` |  | Presents the unavailable housing-server error. | [[Native Map/Shards/S0317 00ab2c80-00ab7380|S0317]] |
| `00ab5590` | `HandleHousingPermissionDenied ⚠downgrade` |  | Presents the house-entry permission error. | [[Native Map/Shards/S0317 00ab2c80-00ab7380|S0317]] |
| `00ab5670` | `HandleHousingZoneNotFound ⚠downgrade` |  | Presents the missing housing-zone error. | [[Native Map/Shards/S0317 00ab2c80-00ab7380|S0317]] |
| `00ab5750` | `HandleHousingZoneLaunchFailure ⚠downgrade` |  | Presents a housing-zone launch error. | [[Native Map/Shards/S0317 00ab2c80-00ab7380|S0317]] |
| `00ab5830` | `HandleHousingZoneFull ⚠downgrade` |  | Explains that a house reached its visitor limit. | [[Native Map/Shards/S0317 00ab2c80-00ab7380|S0317]] |
| `00ab5910` | `SelectHouseFixture` |  | Handles selection of a house fixture. | [[Native Map/Shards/S0317 00ab2c80-00ab7380|S0317]] |
| `00ab5f10` | `LoadHouseRatingKeywords` |  | Loads local house-rating keywords. | [[Native Map/Shards/S0317 00ab2c80-00ab7380|S0317]] |
| `00ab9400` | `FindOrInsertTwoStringRecord` |  | Interning helper for two-string housing link records; callers HandleMessage40LinkRecord (00ac8340) and RebuildListsFromMessageBody (00ac9bd0) carry the housing context in their accepted labels. | [[Native Map/Shards/S0318 00ab73e0-00abcb30|S0318]] |
| `00abb1c0` | `UpdatePlacementSurfaceTarget ⚠downgrade` |  | Per-frame placement surface pick for the held housing preview: chooses between a floor fixture and terrain and records the choice; called from the frame update path 00ac4630 (accepted label HousingFrameUpdate). | [[Native Map/Shards/S0318 00ab73e0-00abcb30|S0318]] |
| `00abd600` | `FixtureDefinitionDtor` | FixtureDefinition | Payload destructor for one fixture definition: it runs when a definition list node is popped (00abf1b0), from the subtype-42 handler 00ac7f40, and from the scalar deleting destructor 00abf010. | [[Native Map/Shards/S0319 00abcb90-00ac2280|S0319]] |
| `00abefc0` | `HousingProcessor::EnsureResourcesLoaded` | HousingProcessor | Loads housing resource data once per processor instance, then refreshes house-related state through the object and virtual callback. The class association is supported by the direct HousingProcessor handler caller. | [[Native Map/Shards/S0319 00abcb90-00ac2280|S0319]] |
| `00abf010` | `FixtureDefinitionScalarDeletingDtor` | FixtureDefinition | Delete form of the fixture-definition record; it has no vtable store of its own, so the record's class comes from the destructor chain 00abd600 -> 00abd310 FixtureAction map. | [[Native Map/Shards/S0319 00abcb90-00ac2280|S0319]] |
| `00abfda0` | `HousingProcessor::ResetLoadedHouse` | HousingProcessor | Resets loaded housing state. The process flag at [this+0x10]+0x64c gates the virtual call; the object at +0x4198 is its receiver, not the tested flag. | [[Native Map/Shards/S0319 00abcb90-00ac2280|S0319]] |
| `00ac4630` | `HousingFrameUpdate` |  | Per-frame housing-controller update: sends a queued preview request, updates local control state, runs edit-mode work, and processes the exploded-fixture loop. | [[Native Map/Shards/S0320 00ac2a10-00ac7f40|S0320]] |
| `00ac8340` | `ReadAndLinkHousingRecord → HousingProcessor::HandleFixtureUpdate` |  | HousingProcessor opcode 0x28 dispatches here. The function applies a parsed fixture-update record to client housing state; exact field meanings remain unresolved. | [[Native Map/Shards/S0321 00ac8340-00acd3f0|S0321]] |
| `00ac8680` | `SendPickupAllFixturesAndOptionalRatingRemoval` |  | Stored housing callback that requests pickup of all fixtures and may also run the house rating removal path. | [[Native Map/Shards/S0321 00ac8340-00acd3f0|S0321]] |
| `00ac9bd0` | `ApplyFixtureItemListMessage` |  | Inbound handler reached from HousingProcessor inner opcode 0x2b; it parses the community-catalogued HousingPacketFixtureItemList and updates associated client records. | [[Native Map/Shards/S0321 00ac8340-00acd3f0|S0321]] |
| `00cb93f0` | `ResolveSubBlock2f0OrDefault` |  | Supplies the placement-vector block (+0x2f0) for the housing placement selection; the vault note says the selection copies its vectors from this function into the processor. | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cb9420` | `ResolveSubBlock190OrDefault` |  | Supplies the second placement-vector block (+0x190) for the housing placement selection; the vault note cites this function next to 00cb93f0 for the processor copy. | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |

## 🟡 speculative (14)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00aab920` | `GetActorUidFromField41A0 ? ⚠rename` |  | Accessor on the housing manager used by the caller 00c65480. | [[Native Map/Shards/S0315 00aa90e0-00aae6b0|S0315]] |
| `00ab39e0` | `CopyFixtureFields ?` |  | Copies a word at source offset 0x7c and embedded fields through offset 0xa4 into the destination when its pointer is non-null. | [[Native Map/Shards/S0317 00ab2c80-00ab7380|S0317]] |
| `00ab43e0` | `FindFixtureItemRow ? ⚠rename` |  | Resolves a fixture row in the housing user-interface table. | [[Native Map/Shards/S0317 00ab2c80-00ab7380|S0317]] |
| `00abb130` | `RemoveKeyedChainEntries ?` |  | Hashes the key argument with FUN_00705b00 and scans the 16-bucket table at this+0x28b0 for the matching entry, then walks the chains linked at node+0xb8 and node+0xb0 and calls FUN_00ab9e30 to remove each entry for whic… | [[Native Map/Shards/S0318 00ab73e0-00abcb30|S0318]] |
| `00ac9140` | `SavePlacementAtPositionWrapper ? ⚠rename` |  | Wrapper used by the placement action caller to position a preview and immediately run the housing save path; that purpose relies on its 00ac8790 callee and mapped caller. | [[Native Map/Shards/S0321 00ac8340-00acd3f0|S0321]] |
| `00cb90a0` | `HousePiece::SetByte25One ?` | HousePiece | Virtual slot 2 of HousePiece (thiscall, one ignored stack argument, ret 4): stores 1 in the byte at receiver+0x25. Bytes c6 41 25 01 c2 04 00. | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cb90b0` | `HousePiece::SetChildByte100IfPresent ?` | HousePiece | Virtual slot 7 of HousePiece (thiscall, one ignored stack argument, ret 4): when the pointer at receiver+0x1c is nonnull, stores 1 in the byte at that pointer plus 0x100. Bytes 8b 41 1c 85 c0 74 07 c6 80 00 01 00 00 c2 … | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cb91d0` | `HousePiece::Slot9ForwardKey70 ?` | HousePiece | Virtual slot 9 of HousePiece and Fixture (shared implementation; thiscall with no stack argument of its own): bytes 8b 51 70 51 8b c4 89 10 8b 49 68 8b 01 8b 50 10 ff d2 c3 push the dword at receiver+0x70 as one stack a… | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cb9a40` | `HousePiece::TailCallSlot9 ?` | HousePiece | Virtual slot 0 of HousePiece and slot 0 of Fixture (shared implementation): bytes 8b 01 8b 50 24 ff e2 load the vtable, load slot 9 (+0x24) and tail-jump to it with ECX = receiver and the caller's stack unchanged. Not a… | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cb9a50` | `HousePiece::AdjustorThunk50 ?` | HousePiece | Adjustor thunk at slot 0 of the secondary HousePiece vftable (018372cc): bytes 83 e9 50 e9 48 03 00 00 subtract 0x50 from ECX and jump to 00cb9da0 (rel32). It forwards to 00cb9da0 with the adjusted receiver. | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cb9a60` | `HousePiece::ClearDword20 ?` | HousePiece | Virtual slot 13 of HousePiece and Fixture (shared implementation): bytes c7 41 20 00 00 00 00 c2 04 00 store zero in the dword at receiver+0x20 and return, popping 4. | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cb9e80` | `HousePiece::SetFlag25ThenLazyCreate ?` | HousePiece | Virtual slot 1 of the secondary HousePiece vftable (018372cc), thiscall with one stack argument: stores 1 at receiver+0x25; when byte +0x24 is nonzero and bit 1 of the byte at the argument+0x10 is clear, calls 00705200,… | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cba4d0` | `Fixture::AdjustorThunk50 ?` | Fixture | Adjustor thunk at slot 0 of the secondary Fixture vftable (01837374): bytes 83 e9 50 e9 68 0b 00 00 subtract 0x50 from ECX and jump to 00cbb040 (rel32). | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cbb060` | `Fixture::Slot1ThenCandidateBuild ?` | Fixture | Virtual slot 1 of the secondary Fixture vftable (01837374), thiscall with one stack argument: stores 1 at receiver+0x25; when byte +0x24 is nonzero and bit 1 of the byte at the argument+0x10 is clear, calls 00705200, 00… | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |

## ⚪ unknown (15)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00ab0a80` | `— ⚠reject` | HousingProcessor | Switches housing edit mode and installs or tears down its event handlers. | [[Native Map/Shards/S0316 00aae6e0-00ab2c20|S0316]] |
| `00ab1380` | `— ⚠reject` |  | The per-frame held-preview caller and the use of FilterPlacementConvexHits bind this transform adjustment to housing placement. | [[Native Map/Shards/S0316 00aae6e0-00ab2c20|S0316]] |
| `00ab1e30` | `— ⚠reject` |  | Builds the housing fixture preview rotation; the rotation constants and action conditions are recorded in the cited note. | [[Native Map/Shards/S0316 00aae6e0-00ab2c20|S0316]] |
| `00ab6110` | `—` |  | Finds a keyword row, updates its stored byte when the requested flag differs, and notifies the observer if present. | [[Native Map/Shards/S0317 00ab2c80-00ab7380|S0317]] |
| `00abb630` | `— ⚠reject` |  | Per-frame update of the held placement preview (the ghost fixture), called from HousingFrameUpdate (00ac4630, accepted label). | [[Native Map/Shards/S0318 00ab73e0-00abcb30|S0318]] |
| `00abd680` | `— ⚠reject` |  | Reads a housing name resource table. The class attribution is not direct in this body; its callers use it in HousingProcessor resource-loading paths. | [[Native Map/Shards/S0319 00abcb90-00ac2280|S0319]] |
| `00abdb20` | `— ⚠reject` |  | Static initialisation hook for the housing content, reached from 00916e70; it is the only site that loads both tables together. | [[Native Map/Shards/S0319 00abcb90-00ac2280|S0319]] |
| `00abf1b0` | `— ⚠reject` |  | Pops and destroys one head fixture-definition node. This feature label rests on direct caller 00abf760 and destructor callee 00abd600, so the name remains likely. | [[Native Map/Shards/S0319 00abcb90-00ac2280|S0319]] |
| `00ac2280` | `— ⚠reject` | HousingProcessor | Member-wise teardown of the housing component built by the constructor 00abf850; reached from its owner's destruction at 00a1d6d0 and from EH unwind funclets. | [[Native Map/Shards/S0319 00abcb90-00ac2280|S0319]] |
| `00ac8790` | `— ⚠reject` |  | Housing editor save action. Note 025 directly identifies this address as the sender for housing subtype 5; its body also confirms the connected guard and selection clear. | [[Native Map/Shards/S0321 00ac8340-00acd3f0|S0321]] |
| `00ac8940` | `— ⚠reject` |  | Housing editor action for placing or saving the held preview fixture. The ordinary new-placement path sends pending subtype 2; a separate debug path can send subtype 1. | [[Native Map/Shards/S0321 00ac8340-00acd3f0|S0321]] |
| `00ac8d90` | `— ⚠reject` |  | Housing editor confirmation state machine: state 1 repositions an existing fixture and invokes the save path; state 2 sends a pickup request and advances to state 3; state 3 sends a place request only when the fixture l… | [[Native Map/Shards/S0321 00ac8340-00acd3f0|S0321]] |
| `00aca330` | `— ⚠reject` |  | Inbound housing handler for dispatcher case 0x1c (28): it builds the client-side house object from the instance-data message and dumps it to the admin console. | [[Native Map/Shards/S0321 00ac8340-00acd3f0|S0321]] |
| `00cb98f0` | `— ⚠reject` | HousePiece | Destructor body of HousePiece: releases its buffers, shared references and embedded string. | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cb9aa0` | `— ⚠reject` | HousePiece | Virtual slot 8 of HousePiece and Fixture (shared implementation; thiscall, one stack argument, ret 4): calls slot 9 of the receiver with one stack argument; when the result is nonnull calls 00777610 on that result with … | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00ac4630` | `HousingFrameUpdate` | send | BaseHousingPacket |  | Hypothesis: the server may answer subtype 20 with data usable by a housing preview. This function proves the client send only; it does not identify the reply subtype. The captured 1-to-42 flow in Housing Placement Responses does not establish a 20-to-42 pair. |
| `00ac8340` | `ReadAndLinkHousingRecord → HousingProcessor::HandleFixtureUpdate` | recv | BaseHousingPacket |  |  |
| `00ac8680` | `SendPickupAllFixturesAndOptionalRatingRemoval` | send | BaseHousingPacket |  |  |
| `00ac8680` | `SendPickupAllFixturesAndOptionalRatingRemoval` | send | PacketTunneledClientPacket |  |  |
| `00ac8680` | `SendPickupAllFixturesAndOptionalRatingRemoval` | send | PacketTunneledClientWorldPacket |  |  |
| `00ac9bd0` | `ApplyFixtureItemListMessage` | recv | HousingPacketFixtureItemList |  | Hypothesis: the original server sends a housing-family subtype-43 item-list body that this client parses. This depends on the 0x2b dispatch at 00aca5d0 and the field reader at 00ac9a90; it does not establish when the original server sent it or that actor nodes are destroyed. |
| `00ac9140` | `SavePlacementAtPositionWrapper ? ⚠rename` | send | BaseHousingPacket |  |  |
| `00ac9140` | `SavePlacementAtPositionWrapper ? ⚠rename` | send | PacketTunneledClientPacket |  |  |
| `00ac8790` | `— ⚠reject` | send | BaseHousingPacket |  |  |
| `00ac8790` | `— ⚠reject` | send | PacketTunneledClientPacket |  |  |
| `00ac8940` | `— ⚠reject` | send | BaseHousingPacket |  |  |
| `00ac8940` | `— ⚠reject` | send | PacketTunneledClientPacket |  |  |
| `00ac8d90` | `— ⚠reject` | send | BaseHousingPacket |  |  |
| `00ac8d90` | `— ⚠reject` | send | PacketTunneledClientPacket |  |  |
| `00aca330` | `— ⚠reject` | recv | HousingPacketInstanceData |  | Hypothesis: a housing subtype-28 instance-data message is what activates and initializes a house on first client receipt. This states the client-side path only; it depends on the dispatcher at 00aca5d0 and the body parser at 00ac9dd0, while the packet name comes from the community catalogue. |
