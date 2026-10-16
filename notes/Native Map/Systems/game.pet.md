# System: game.pet

80 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (27)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00afd610` | `ControllerPet::~ControllerPet` | ControllerPet | Tears down the ControllerPet state implementation and delegates to the base teardown 00afe290. | [[Native Map/Shards/S0331 00afb7a0-00b005c0|S0331]] |
| `00afd6f0` | `ControllerPet::ControllerPet` | ControllerPet | Constructs the ControllerPet state implementation. | [[Native Map/Shards/S0331 00afb7a0-00b005c0|S0331]] |
| `00afd7b0` | `ControllerPet::ScalarDeletingDtor` | ControllerPet | Destroys a ControllerPet and frees its storage on the delete path. | [[Native Map/Shards/S0331 00afb7a0-00b005c0|S0331]] |
| `00b562e0` | `LoadPetEffectsFile` |  | Loads pet-effect configuration entries from the resource file. | [[Native Map/Shards/S0347 00b56090-00b5b410|S0347]] |
| `00b589c0` | `PetsHandler::PetExpireTime` | PetsHandler | Updates the pet trial countdown and its client-facing display. | [[Native Map/Shards/S0347 00b56090-00b5b410|S0347]] |
| `00b58bd0` | `PetsHandler::PetIsActive` | PetsHandler | Updates active-pet state and reports the active identifier to the client-side script layer. | [[Native Map/Shards/S0347 00b56090-00b5b410|S0347]] |
| `00b58d90` | `EquipPetMannequinBundle` |  | Equips mannequin bundle data on pet actors; the operation name is directly present in its error message. | [[Native Map/Shards/S0347 00b56090-00b5b410|S0347]] |
| `00b5ae90` | `LoadDefaultPetNamesFromResources` |  | Loads the pet-name table from the default resource directory. | [[Native Map/Shards/S0347 00b56090-00b5b410|S0347]] |
| `00b5b1b0` | `BasePetTrickDataSource::ScalarDeletingDtor` | BasePetTrickDataSource | Implements the scalar deleting destructor entry for BasePetTrickDataSource. | [[Native Map/Shards/S0347 00b56090-00b5b410|S0347]] |
| `00b5b210` | `PetTricksDataSource::PetTricksDataSource` | PetTricksDataSource | Constructs the pet tricks data source derived from the common pet-trick table. | [[Native Map/Shards/S0347 00b56090-00b5b410|S0347]] |
| `00b5b2e0` | `PetTricksDataSource::ScalarDeletingDtor` | PetTricksDataSource | Implements the scalar deleting destructor entry for PetTricksDataSource. | [[Native Map/Shards/S0347 00b56090-00b5b410|S0347]] |
| `00b5b340` | `PetTricksEquippedDataSource::PetTricksEquippedDataSource` | PetTricksEquippedDataSource | Constructs the equipped pet-trick data source. | [[Native Map/Shards/S0347 00b56090-00b5b410|S0347]] |
| `00b5b410` | `PetTricksEquippedDataSource::ScalarDeletingDtor` | PetTricksEquippedDataSource | Implements the scalar deleting destructor entry for PetTricksEquippedDataSource. | [[Native Map/Shards/S0347 00b56090-00b5b410|S0347]] |
| `00cc7080` | `InitializePetTrickUnlockState` |  | Initializes trick-unlock state used by the pet gesture handler. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00cc7420` | `UnlockPetTricksAfterGesture` |  | Pet gesture completion path that unlocks tricks or restores the game dock after a pet error. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00cc7930` | `PetGestureDtor` | PetGesture | Destroys PetGesture state and its owned arrays and effects. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00cc8760` | `PetGestureScalarDeletingDtor` | PetGesture | MSVC scalar deleting destructor for PetGesture. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00cc8780` | `GestureScaleRatio` |  | Normalizes gesture size as a scoring component. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00cc8890` | `ScorePetTrickGesture` |  | Grades a completed pet trick drawing and prepares its client result UI. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00cc8d20` | `AdvancePetGestureState` |  | Runs PetGesture transitions and client notifications. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00cc9020` | `PetGestureCtor` | PetGesture | Constructs client gesture state and loads its settings. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00cc9330` | `BeginPetGestureRound` |  | Begins display and capture setup for a pet gesture round. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00cc9a90` | `FinishPetGestureState` |  | Cleans up after a gesture and restores pet UI state. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00cc9bb0` | `LoadPetGestureDataAndSettings` | PetGesture | Loads shipped gesture geometry and configurable scoring/runtime settings. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00cce850` | `PetTactile::Ctor` | PetTactile | Constructs pet tactile interaction state and its spot map. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00cce940` | `PetTactile::ScalarDeletingDtor` | PetTactile | MSVC scalar deleting destructor for PetTactile. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `01044560` | `CharacterBase::CreatePetRecord` | CharacterBase | Creates a new pet record for this owner with the next pet id, repairing the id counter at this+0x164 if it is behind the existing records. | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |

## 🟢 likely (23)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0097c5c0` | `PetNameTemplateExpand` |  | Expands the PetName template for a pet, using the pet_full_name key. | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `00a127e0` | `PetsHandler_PetCanDraw_Thunk` |  | Thin wrapper to the PetsHandler::PetCanDraw implementation. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00b56090` | `InitTwoStringFieldsAndValue` |  | Called by array helpers that create pet-name records; record meaning beyond the two string fields remains unknown. | [[Native Map/Shards/S0347 00b56090-00b5b410|S0347]] |
| `00b56920` | `UnlockUtilitiesResetB → PetsHandler::UnlockUtilities` |  | Resets pet utility-unlock state; the extra callback's role is unknown. | [[Native Map/Shards/S0347 00b56090-00b5b410|S0347]] |
| `00b56dd0` | `CreateActorForMannequinRecord` |  | Creates the actor associated with a mannequin-like record in the pet handler path; the created actor's feature role is unresolved. | [[Native Map/Shards/S0347 00b56090-00b5b410|S0347]] |
| `00b57170` | `CreateActorFromPetTargetRecord` |  | Creates or refreshes an actor for a pet target record; the exact record type is inferred from its pet-handler callers. | [[Native Map/Shards/S0347 00b56090-00b5b410|S0347]] |
| `00b59070` | `UnlockPetTricks` |  | Resets and announces the pet trick-unlock state. | [[Native Map/Shards/S0347 00b56090-00b5b410|S0347]] |
| `00b596f0` | `InitializePetClientBindings` |  | Initializes the pet client handler and binds its pet list/active state into Lua. | [[Native Map/Shards/S0347 00b56090-00b5b410|S0347]] |
| `00b59b90` | `DestroyPetClientHandlerState` |  | Tears down the pet client-handler state initialized by the adjacent constructor; this class association is based on matching fields/call paths. | [[Native Map/Shards/S0347 00b56090-00b5b410|S0347]] |
| `00b5ce60` | `PetsHandler::HandlePetPacket ⚠downgrade` | PetsHandler | The switch handles numerous pet reply and event subtypes; observed branches include favorite trick updates, trick success, pet status, name rejection, trial expiration, and pet utility events. | [[Native Map/Shards/S0348 00b5b470-00b5ff50|S0348]] |
| `00b5fb30` | `DispatchPetActionRecord` |  | Dispatches an enabled action selected from the pet's action records; the exact non-toy action semantics are not identified here. | [[Native Map/Shards/S0348 00b5b470-00b5ff50|S0348]] |
| `00b60770` | `PetTileTargetHandler::OnMessage` | PetTileTargetHandler | Virtual handler entry (slot 3) of the pet tile-target handler that routes two message kinds to their implementations; FUN_00b5fcf0 is the client follow/pet-can-go-location path per the pet notes. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00cc6710` | `SetFlag138AndNotifyForState14` |  | Its recorded callers are pet-gesture state handlers; this function's own body implements the state-14 guarded sequence described here. | [[Native Map/Shards/S0415 00cc1a00-00cc6bc0|S0415]] |
| `00cc6a80` | `FindState14ObjectByPair` |  | A pet-gesture update caller uses the matching object as its active actor candidate; this function only proves the state and pair comparison. | [[Native Map/Shards/S0415 00cc1a00-00cc6bc0|S0415]] |
| `00cc6af0` | `ClearFlag138AndResetDependentState` |  | Recorded callers are pet trick initialization and gesture completion handlers; this helper resets their shared state fields. | [[Native Map/Shards/S0415 00cc1a00-00cc6bc0|S0415]] |
| `00cc81f0` | `CompareGestureSegmentDirections` |  | Computes directional similarity between normalized gesture paths. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00cc8630` | `RegisterGestureSamplePoints` |  | Registers sampled gesture positions for later proximity and effect processing. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00cc97f0` | `UpdateGestureRoundState` |  | Updates gesture startup and timeout state. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00cc9950` | `SampleGestureRoundState` |  | Processes the timed gesture sampling phase. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00ccdda0` | `PetsHandler::ResetPetPanel` | PetsHandler | Resets pet panel state after dismissal or pet-panel reset. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00cce490` | `PetTactile::ReleaseSpotEffects` | PetTactile | Releases visual effects created for pet tactile grooming spots. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00cce620` | `PetTactile::Dtor ⚠downgrade` | PetTactile | Destroys the pet tactile activity and its grooming map. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00cce960` | `PetTactile::RefreshSpots` | PetTactile | Refreshes tactile grooming spots and their associated map entries. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |

## 🟡 speculative (15)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009652a0` | `AssignStringAndRefreshTransform ? ⚠rename` |  | The confirmed PetsHandler caller supplies the pet-related context; the string and transform fields' meanings are not identified. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `00afd680` | `ControllerPet::GetByte3b4 ?` | ControllerPet | Thiscall getter, no args, vtable slot 72 (+0x120): returns the byte at this+0x3b4 (mov al,[ecx+0x3b4]; ret). | [[Native Map/Shards/S0331 00afb7a0-00b005c0|S0331]] |
| `00afd690` | `ControllerPet::SetByte3b4 ?` | ControllerPet | Thiscall setter with one byte arg (ret 4), vtable slot 73 (+0x124): stores the arg byte into this+0x3b4. | [[Native Map/Shards/S0331 00afb7a0-00b005c0|S0331]] |
| `00afd6c0` | `ControllerPet::InvokeSlot53ThenSlot45 ?` | ControllerPet | Thiscall, vtable slot 46 (+0xb8). Calls this->vtable[+0xd4] with one stack arg this+0xe0, then this->vtable[+0xb4] with one stack arg 0. | [[Native Map/Shards/S0331 00afb7a0-00b005c0|S0331]] |
| `00afd810` | `ControllerPet::DistanceOfFourFloatsToSlot5c ?` | ControllerPet | Thiscall with one arg, vtable slot 53 (+0xd4). Calls 0094d3b0(arg), this->vtable[+0x70](DAT_017eb654), this->vtable[+0xdc](1), 008a6660(), 00b536e0() to get an object whose vtable[+0x24] fills a 4-float out buffer, take… | [[Native Map/Shards/S0331 00afb7a0-00b005c0|S0331]] |
| `00afd8d0` | `ControllerPet::ForwardListNodes_c8 ?` | ControllerPet | Thiscall with one arg, vtable slot 50 (+0xc8): calls 00afe5d0(arg), then this->vtable[+0x128](arg), then walks the singly linked list headed at this+0xc8 (next pointer at node+0x04), calling 0094c8b0() on each node and … | [[Native Map/Shards/S0331 00afb7a0-00b005c0|S0331]] |
| `00bda800` | `AccessorThenTailJump_00b53690 ?` |  | The target label suggests a pet-entity refresh, but that interpretation depends on the target function and remains speculative. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00cc7f60` | `AverageVector2Points ?` |  | Calculates the center of a gesture point cloud. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00cc8040` | `MaxAxisSpan ?` |  | Measures the larger bounding-box dimension of a gesture. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00cc8ff0` | `AdvanceToGestureReadyState ?` |  | Applies one pet-gesture notification state transition. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00cca3f0` | `PetGestureSetTerminalState ? ⚠rename` | PetGesture | Sets gesture object to terminal state eight. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00cca480` | `DispatchPetGestureUpdate ? ⚠downgrade` |  | Central update dispatcher for PetGesture state machine. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00ccdc90` | `ReadPetUiMode12Object ?` |  | Returns a nested pet UI object when its runtime type id equals 12. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00ccdcc0` | `SetPetObjectFlagZero ?` |  | Updates a field on the nested pet UI object when its runtime type matches. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00ccdd20` | `SetPetObjectFlagOne ?` |  | Updates a field on the nested pet UI object when its runtime type matches. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |

## ⚪ unknown (15)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0097b770` | `— ⚠reject` |  | Formats a pet name from its first and second name parts using the pet_first and pet_second template keys. | [[Native Map/Shards/S0261 00976740-0097c910|S0261]] |
| `00afd920` | `— ⚠reject` | ControllerPet | Thiscall with one int arg (time step), vtable slot 74 (+0x128). Calls 0094d650() and returns if null. Updates the counter at +0xec by the arg and flag byte +0x3b5 from flags +0x219, +0x85 and +0xec (thresholds 0x1f4, 0x… | [[Native Map/Shards/S0331 00afb7a0-00b005c0|S0331]] |
| `00b56850` | `— ⚠reject` |  | Resets utility-unlock state in the pet handler; method identity is inferred from the function's log label and handler caller. | [[Native Map/Shards/S0347 00b56090-00b5b410|S0347]] |
| `00b56a00` | `— ⚠reject` |  | Resets pet utility-unlock state; the final cleanup target is a stub in the export. | [[Native Map/Shards/S0347 00b56090-00b5b410|S0347]] |
| `00b576e0` | `— ⚠reject` |  | Provides indexed pet-name records for the PetNames table helpers. | [[Native Map/Shards/S0347 00b56090-00b5b410|S0347]] |
| `00b5aa40` | `— ⚠reject` |  | Loads the client pet-name table used for pet naming. | [[Native Map/Shards/S0347 00b56090-00b5b410|S0347]] |
| `00b5b120` | `— ⚠reject` | BasePetTrickDataSource | Constructs the base pet trick data source. | [[Native Map/Shards/S0347 00b56090-00b5b410|S0347]] |
| `00b604b0` | `— ⚠reject` | PetsHandler | Keeps the handler's tracked pet-target pair in sync with the current target and tells the server the client is previewing that pet. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b60610` | `— ⚠reject` | PetsHandler | Client path for a server-forced pet dismissal: it tells the server the pet was recalled, notifies the Lua/UI side and drops the local active-pet state. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00cc7a00` | `— ⚠reject` |  | Normalizes sample and player gesture paths for comparison. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00cc8400` | `— ⚠reject` |  | Combines scale, center, segment, and timing measurements for gesture grading. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00cc95f0` | `— ⚠reject` |  | Processes pointer movement during pet gesture drawing. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00cca410` | `— ⚠reject` |  | Cancels active gesture and runs completion cleanup. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00ccdc70` | `— ⚠reject` |  | Pet camera or panel-side-effect helper; exact callee behavior is not identified here. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00cce1d0` | `— ⚠reject` |  | Maintains pet tactile or grooming activity state; this role is inferred from its callers and PetGroomSpot-shaped fields. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00b5ce60` | `PetsHandler::HandlePetPacket ⚠downgrade` | recv | PacketTunneledClientPacket |  | Hypothesis: the absent server emits tunneled pet-family response bodies that drive these client state and UI updates. This depends on the client's subtype readers and the community pet-family enum; reconstructed emulator semantics do not prove original server policy. |
