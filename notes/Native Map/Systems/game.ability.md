# System: game.ability

39 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (12)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a2d910` | `AbilityTerrainClickHandler::MatchKind` | AbilityTerrainClickHandler | Slot-2 predicate of AbilityTerrainClickHandler; the meaning of the accepted kind values is not established. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a2ef20` | `AbilityActorClickHandler::AbilityActorClickHandler` | AbilityActorClickHandler | Constructor of the click handler that resolves a clicked actor as an ability target; built by the ability processor 00a324e0. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a2ef80` | `TargetProcessorEventHandler::TargetProcessorEventHandler` | TargetProcessorEventHandler | Base-event-handler constructor (one of three identical emissions: 00a2ef80, 00a2f0b0 and 00a2f740) used by the ability click/target handlers. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a2f030` | `AbilityPlayerClickHandler::AbilityPlayerClickHandler` | AbilityPlayerClickHandler | Constructor of the click handler that resolves a clicked player as an ability target; built by the ability processor 00a324e0. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a2f090` | `AbilityPlayerClickHandler::MatchKind` | AbilityPlayerClickHandler | Slot-2 predicate of AbilityPlayerClickHandler; the semantic meaning of the mask bits is not established. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a2f6e0` | `AbilityTerrainClickHandler::AbilityTerrainClickHandler` | AbilityTerrainClickHandler | Constructor of the click handler that resolves a clicked terrain point as an ability target; the base is the tile-handler ctor and 00a324e0 is the ability processor that builds the handlers. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a324e0` | `AbilityProcessor::AbilityProcessor` | AbilityProcessor | Constructs AbilityProcessor, which owns the ability-definition table and the terrain, player and actor click handlers. | [[Native Map/Shards/S0293 00a2fef0-00a35000|S0293]] |
| `00a355f0` | `AbilityProcessor::SendIfFlagAt24` | AbilityProcessor | Forced-send trigger for the ability processor: a set +0x24 flag makes it issue the start-ability request through 00a35150. | [[Native Map/Shards/S0294 00a35150-00a39a80|S0294]] |
| `00a35920` | `AbilityTerrainClickHandler::DispatchEventToProcessor` | AbilityTerrainClickHandler | Forwards terrain-click events to the ability processor owned at +0xc: a click-target vector is stored and sent on code 0x8. | [[Native Map/Shards/S0294 00a35150-00a39a80|S0294]] |
| `00a359a0` | `AbilityActorClickHandler::DispatchEventToProcessor` | AbilityActorClickHandler | Actor-click counterpart of the terrain handler: a click vector is stored on the ability processor and the ability start request is sent on code 0x8. | [[Native Map/Shards/S0294 00a35150-00a39a80|S0294]] |
| `00a35a40` | `AbilityPlayerClickHandler::DispatchEventToProcessor` | AbilityPlayerClickHandler | Player-click counterpart of the terrain and actor click handlers: a player click can start the ability send through 00a358b0 and sets the processor's gating byte from the clicked player's state. | [[Native Map/Shards/S0294 00a35150-00a39a80|S0294]] |
| `00a35cc0` | `AbilityProcessor::HandleAbilityPacketBySubOpcode` | AbilityProcessor | Receive-side dispatcher for the ability packet family on the client: reads the sub-opcode, parses the matching ability packet and applies it to the ability processor and the ActionBar data. | [[Native Map/Shards/S0294 00a35150-00a39a80|S0294]] |

## 🟢 likely (17)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a2d790` | `ClearStateFlagsAndUnlinkHandles` |  | State teardown shared by the state entry points 00a2d820, 00a2e620 and 00a2e6b0; FUN_00acb430 is a container unlink (bucket walk, count-- at +8, node links at +4/+8), not an effect API. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a2d950` | `AbilityActorClickHandler::MatchKind ⚠downgrade` | AbilityActorClickHandler | Slot-2 predicate of AbilityActorClickHandler; it has the same body as the terrain-handler predicate at 00a2d910, while the class binding comes directly from its own vtable slot. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a2da50` | `ConstructAbilityDefinitionWithKey` |  | Initialises one AbilityDefinition node before it is linked into the 100-bucket map; caller 00a31320 is the ability definition store path. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a2efd0` | `AbilityActorClickHandler::ScalarDeletingDtor` | AbilityActorClickHandler | MSVC scalar-deleting destructor of the actor click handler built by the constructor 00a2ef20. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a2f0b0` | `TargetProcessorEventHandler::TargetProcessorEventHandler ⚠downgrade` | TargetProcessorEventHandler | Base-event-handler constructor (one of three identical emissions: 00a2ef80, 00a2f0b0 and 00a2f740) used by the ability click/target handlers. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a2f100` | `AbilityPlayerClickHandler::ScalarDeletingDtor` | AbilityPlayerClickHandler | MSVC scalar-deleting destructor of the player click handler built by the constructor 00a2f030. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a2f160` | `CallResetIfFlagSet` |  | Guarded wrapper around the reset 00a2e620; no anchor in this function names a feature, so the name states the guard only. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a2f740` | `TargetProcessorEventHandler::TargetProcessorEventHandler ⚠downgrade` | TargetProcessorEventHandler | Base-event-handler constructor (one of three identical emissions: 00a2ef80, 00a2f0b0 and 00a2f740) used by the ability click/target handlers. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a2f790` | `AbilityTerrainClickHandler::ScalarDeletingDtor` | AbilityTerrainClickHandler | MSVC scalar-deleting destructor of the terrain click handler built by the constructor 00a2f6e0. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a30870` | `AbilityValidatePlayerStatus` |  | Gate for ability validation: refuses the start when the player status bits overlap the mask, and logs the status. Called from the ability start path 00a35150 and from 00a35600 and 00a36a70. | [[Native Map/Shards/S0293 00a2fef0-00a35000|S0293]] |
| `00a30d20` | `AbilityEffectNamedFields` |  | Named-field table for one ability effect, built for the effect rows in 00a34e30. | [[Native Map/Shards/S0293 00a2fef0-00a35000|S0293]] |
| `00a34c80` | `SendPurchaseAbility` |  | Client purchase-ability send invoked from the Lua PurchaseAbility handler 00c0a6c0. | [[Native Map/Shards/S0293 00a2fef0-00a35000|S0293]] |
| `00a34d10` | `AbilityDefinitionLookupOrRequest` |  | Ability-definition cache lookup: returns the node when loaded, otherwise requests it from the server once. | [[Native Map/Shards/S0293 00a2fef0-00a35000|S0293]] |
| `00a34e00` | `AbilityDefinitionNamedFieldsByKey` |  | Returns the named-field table of an ability definition by id for the item abilities list. | [[Native Map/Shards/S0293 00a2fef0-00a35000|S0293]] |
| `00a34e30` | `AbilityEffectRowsForItem` |  | Builds the ability effect rows for an item's ability list. | [[Native Map/Shards/S0293 00a2fef0-00a35000|S0293]] |
| `00a35150` | `AbilityProcessor::SendStartAbilityRequest` | AbilityProcessor | Common send path for an ability start request from the local player's ability bar (cast/activate), per the vault note on food/potion activation. | [[Native Map/Shards/S0294 00a35150-00a39a80|S0294]] |
| `00a35600` | `AbilityProcessor::DispatchTargetModeAndSend ⚠downgrade` | AbilityProcessor | Cast target resolution for an ability-bar activation: the self flag is forced for selector 1 with target selector 2 or 4, and other target selectors copy the owner's position/target pairs before the common send. | [[Native Map/Shards/S0294 00a35150-00a39a80|S0294]] |

## 🟡 speculative (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a35810` | `StoreVector16AndSend ?` |  | If the byte-owner object at this+4 has a non-null +0x1a8 pointer: calls 00acb430 with this+0x74 (accepted label HashListRemoveElement_7c), sets byte +0x61 of the +0x1a8 object to 1, copies the four dwords of param_2 int… | [[Native Map/Shards/S0294 00a35150-00a39a80|S0294]] |
| `00a35860` | `StoreVector16FromPtrAndSend ?` |  | If the owner object at this+4 has a non-null +0x1a8 pointer: calls 00acb430 with this+0x74 (accepted label HashListRemoveElement_7c), copies the four dwords at param_3 into this+0x40..+0x4c (param_2 is unused), then cal… | [[Native Map/Shards/S0294 00a35150-00a39a80|S0294]] |
| `00a358b0` | `StoreSlot10PairAndPositionThenSend ?` |  | Thiscall; param_2 is an object pointer. If this->+4->+0x1a8 is non-null: calls vtable slot +0x10 of the object at param_2+0x54 with an out buffer (two dwords), stores the first dword at this+0x30 and the second at this+… | [[Native Map/Shards/S0294 00a35150-00a39a80|S0294]] |

## ⚪ unknown (7)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a2d820` | `— ⚠reject` |  | Mechanical flag/router helper called from 00a35600 and 00a35cc0; the mode values and the three callees are unidentified, so the name states only what the code does. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a2e620` | `— ⚠reject` |  | State reset used by eight call sites (009cf430, 00a35150, 00a35600, 00a36a70 and others); no anchor in this function names a feature, so the name states the mechanism only. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a2e6b0` | `— ⚠reject` |  | Arm-and-classify step of the state switch: it sets one flag, routes on the mode dword and classifies an actor id pair using the disposition field at +0x178 that note 053 documents for the tab-target selector. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a309c0` | `— ⚠reject` |  | Named-field table for an ability definition (used by 00a34e00). | [[Native Map/Shards/S0293 00a2fef0-00a35000|S0293]] |
| `00a31860` | `— ⚠reject` | AbilityProcessor | Destructor of AbilityProcessor, the per-player ability request processor with click handlers. | [[Native Map/Shards/S0293 00a2fef0-00a35000|S0293]] |
| `00a33760` | `— ⚠reject` | AbilityProcessor | Handler for ability packet sub-opcode 4 (AbilityPacketLaunchAndLand), reached from dispatcher case 4 through vtable slot 3. | [[Native Map/Shards/S0293 00a2fef0-00a35000|S0293]] |
| `00a36a70` | `— ⚠reject` | AbilityProcessor | Activation step for an action-bar slot selected by the client's toolbar: it loads the slot's ability/target fields onto the processor and hands the target to the send dispatcher. | [[Native Map/Shards/S0294 00a35150-00a39a80|S0294]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00a355f0` | `AbilityProcessor::SendIfFlagAt24` | send | AbilityPacketClientRequestStartAbility |  |  |
| `00a35920` | `AbilityTerrainClickHandler::DispatchEventToProcessor` | send | AbilityPacketClientRequestStartAbility |  |  |
| `00a359a0` | `AbilityActorClickHandler::DispatchEventToProcessor` | send | AbilityPacketClientRequestStartAbility |  |  |
| `00a35a40` | `AbilityPlayerClickHandler::DispatchEventToProcessor` | send | AbilityPacketClientRequestStartAbility |  |  |
| `00a35cc0` | `AbilityProcessor::HandleAbilityPacketBySubOpcode` | recv | AbilityPacketFailed |  |  |
| `00a35cc0` | `AbilityProcessor::HandleAbilityPacketBySubOpcode` | recv | AbilityPacketSetDefinition |  |  |
| `00a35cc0` | `AbilityProcessor::HandleAbilityPacketBySubOpcode` | recv | AbilityPacketAbilityDefinition |  |  |
| `00a35150` | `AbilityProcessor::SendStartAbilityRequest` | send | AbilityPacketClientRequestStartAbility |  |  |
| `00a35600` | `AbilityProcessor::DispatchTargetModeAndSend ⚠downgrade` | send | AbilityPacketClientRequestStartAbility |  |  |
| `00a36a70` | `— ⚠reject` | send | AbilityPacketClientRequestStartAbility |  |  |
