# System: world.actor

65 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (31)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00788080` | `Actor::~Actor` | Actor | Actor destructor body (non-deleting); called by the scalar deleting destructor 0078b2c0. | [[Native Map/Shards/S0169 00787e70-0078b5f0|S0169]] |
| `00788990` | `Actor::AdjustorDeletingDtor` | Actor | Adjustor thunk: entry for the secondary Actor base's scalar deleting destructor. | [[Native Map/Shards/S0169 00787e70-0078b5f0|S0169]] |
| `0078aa90` | `Actor::Actor` | Actor | Actor constructor: installs the AssetRequester and Actor vftables and builds the Actor's member containers. | [[Native Map/Shards/S0169 00787e70-0078b5f0|S0169]] |
| `0078b2c0` | `Actor::DeletingDtor` | Actor | Scalar deleting destructor of Actor; reached through the secondary-base thunk 00788990. | [[Native Map/Shards/S0169 00787e70-0078b5f0|S0169]] |
| `0078b5f0` | `Actor::Update` | Actor | Per-frame Actor update (virtual slot [2]): timers, camera lookup, transforms, child bounds and sub-updates. | [[Native Map/Shards/S0169 00787e70-0078b5f0|S0169]] |
| `00798db0` | `ActorHandler::Dtor` | ActorHandler | Base ActorHandler destructor body (vtable reset), reached from 28 EH cleanup funclets. | [[Native Map/Shards/S0172 00798490-0079d890|S0172]] |
| `007993d0` | `ActorGroupRequestorDtorBody` | ActorGroupRequestor | Destructor body of ActorGroupRequestor; reached through its scalar deleting destructor 007995d0. | [[Native Map/Shards/S0172 00798490-0079d890|S0172]] |
| `007995d0` | `ActorGroupRequestorScalarDtor` | ActorGroupRequestor | Scalar deleting destructor of ActorGroupRequestor; the body is 007993d0 in this shard. | [[Native Map/Shards/S0172 00798490-0079d890|S0172]] |
| `007997a0` | `ActorGroupRequestorOnAssetLoadFailed` | ActorGroupRequestor | Load-failure handler for asset requests of ActorGroupRequestor: logs the failure and notifies the object at +0x18. | [[Native Map/Shards/S0172 00798490-0079d890|S0172]] |
| `0079c390` | `ActorManagerInterfaceDtorBody` | ActorManagerInterface | Base ActorManagerInterface destructor body (vtable reset), reached from 2 EH cleanup funclets. | [[Native Map/Shards/S0172 00798490-0079d890|S0172]] |
| `0079c400` | `ActorManagerInterfaceScalarDtor` | ActorManagerInterface | Scalar deleting destructor of ActorManagerInterface (vtable slot [0]). | [[Native Map/Shards/S0172 00798490-0079d890|S0172]] |
| `0079c820` | `ActorManagerRecordDrawnActor` | ActorManager | Scene draw statistics for ActorManager: counts drawn and skipped actors and logs their locations. | [[Native Map/Shards/S0172 00798490-0079d890|S0172]] |
| `0079d450` | `ActorManagerDtorBody` | ActorManager | Destructor body of ActorManager: stores the derived vtable, tears down its pools and trees, then resets to the ActorManagerInterface base vtable. | [[Native Map/Shards/S0172 00798490-0079d890|S0172]] |
| `008b8420` | `ActorSockets::ActorSockets` | ActorSockets | Initializes ActorSockets state and its fixed-size lookup tables. | [[Native Map/Shards/S0223 008b7790-008bcf40|S0223]] |
| `009706d0` | `ProxiedObject::SubobjectAt20` | ProxiedObject | Shared virtual accessor used by ProxiedObject and derived actor classes listed in the same vtable slot. | [[Native Map/Shards/S0259 0096e490-00972d30|S0259]] |
| `009706e0` | `ProxiedObject::ReturnFirstStackArgument` | ProxiedObject | Shared stack-argument forwarding implementation in slot 11 across ProxiedObject and derived actor vtables. | [[Native Map/Shards/S0259 0096e490-00972d30|S0259]] |
| `009706f0` | `ProxiedObject::GetField48` | ProxiedObject | Shared virtual field getter for ProxiedObject and the derived actor vtables listed at slot 12. | [[Native Map/Shards/S0259 0096e490-00972d30|S0259]] |
| `00970700` | `ProxiedObject::SubobjectAt4C` | ProxiedObject | Shared virtual accessor used in slot 14 by ProxiedObject and several derived actor classes. | [[Native Map/Shards/S0259 0096e490-00972d30|S0259]] |
| `00beff10` | `ProxiedObject::SetTransform` | ProxiedObject | Updates the shared transform fields used by proxied objects, vehicles and related world entities. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00beff50` | `ProxiedObject::SetOverheadValue` | ProxiedObject | Updates a shared overhead display value for proxied world objects. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00bf00e0` | `ProxiedObject::Ctor` | ProxiedObject | Constructs the common base state for proxied objects. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00bf0240` | `ProxiedObject::Dtor` | ProxiedObject | Releases the common proxied object string during destruction. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00bf0330` | `ProxiedObject::ScalarDeletingDtor` | ProxiedObject | Implements scalar deleting destruction for proxied objects. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00bf07c0` | `ProxiedObject::SpawnOverheadEffect` | ProxiedObject | Creates an overhead effect for the proxied object using direction and state inputs. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00bf0a30` | `ProxiedObject::SpawnOverheadEffectVariant` | ProxiedObject | Creates a second overhead effect variant for the proxied object. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00bf0c10` | `ProxiedObject::SpawnOverheadEffectWithParams` | ProxiedObject | Creates a parameterized overhead effect on proxied objects. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00bfe0f0` | `PlayerSphereTrackable::Ctor` | PlayerSphereTrackable | Initializes the PlayerSphereTrackable base object. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00bfe210` | `PlayerTrackable::CtorWithPayload` | PlayerTrackable | Initializes a PlayerTrackable object with a payload pointer and base state. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00bfe280` | `PlayerTrackable::ScalarDeletingDtor` | PlayerTrackable | Implements the scalar deleting destructor for PlayerTrackable. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00bfe340` | `PlayerSphereTrackable::CtorWithPayload` | PlayerSphereTrackable | Initializes a PlayerSphereTrackable object with a payload pointer and base state. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00bfe3b0` | `PlayerSphereTrackable::ScalarDeletingDtor` | PlayerSphereTrackable | Implements the scalar deleting destructor for PlayerSphereTrackable. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |

## 🟢 likely (17)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00798e80` | `ActorHandler::ReleaseIString` | ActorHandler | Shared vtable slot [6] for ActorHandler, AppModelEntity, ProxiedCharacter, ShipCombatProcessor, ControllerCinematicActorHandler, InspectProxy, PlayerSpawnProjectileEvent, ProxiedVehicle, ProxiedVehicleDemolitionDerby, H… | [[Native Map/Shards/S0172 00798490-0079d890|S0172]] |
| `007994b0` | `ActorGroupRequestorSetFlag28` | ActorGroupRequestor | Setter of the byte flag at +0x28 whose getter is slot [5] (007994c0). | [[Native Map/Shards/S0172 00798490-0079d890|S0172]] |
| `007994c0` | `ActorGroupRequestorGetFlag28` | ActorGroupRequestor | Getter of the byte flag at +0x28 set by slot [4] (007994b0). | [[Native Map/Shards/S0172 00798490-0079d890|S0172]] |
| `0079a880` | `ActorGroupRequestorAcceptAsset` | ActorGroupRequestor | Accepts an AssetManagement::Asset of type ActorGroupDefinitionAsset for this requestor and starts its processing through 00799a30. | [[Native Map/Shards/S0172 00798490-0079d890|S0172]] |
| `0079c210` | `ActorGroupRequestorCreateAsset` | ActorGroupRequestor | Factory slot of ActorGroupRequestor that creates an ActorGroupDefinitionAsset for the request. | [[Native Map/Shards/S0172 00798490-0079d890|S0172]] |
| `0079c460` | `ActorManagerGetFloat1060` | ActorManager | Getter of the float at +0x1060 that slot [9] (0079c950) sets. | [[Native Map/Shards/S0172 00798490-0079d890|S0172]] |
| `0079c470` | `ActorManagerGetByte1064` | ActorManager | Getter of the byte flag at +0x1064 for ActorManager. | [[Native Map/Shards/S0172 00798490-0079d890|S0172]] |
| `0079c7c0` | `ActorManagerFindById` | ActorManager | Looks up an entry by id in ActorManager's bucket table. | [[Native Map/Shards/S0172 00798490-0079d890|S0172]] |
| `0079c7f0` | `ActorManagerFindByIdAlt` | ActorManager | Second vtable entry (slot [3]) of ActorManager with the same id lookup as slot [4]. | [[Native Map/Shards/S0172 00798490-0079d890|S0172]] |
| `0079c950` | `ActorManagerSetFloat1060Clamped` | ActorManager | Setter of the float at +0x1060 read by slot [8] (0079c460). | [[Native Map/Shards/S0172 00798490-0079d890|S0172]] |
| `0079cd90` | `ActorManagerRemoveById` | ActorManager | Removes an actor entry by id and returns it to the ActorManager pool. | [[Native Map/Shards/S0172 00798490-0079d890|S0172]] |
| `0079d5e0` | `ActorManagerSetByte1064` | ActorManager | Setter of the byte at +0x1064 read by slot [7] (0079c470). | [[Native Map/Shards/S0172 00798490-0079d890|S0172]] |
| `0079d5f0` | `ActorManagerCreateEntry` | ActorManager | Creates an actor entry from a request and registers it in the ActorManager tables. | [[Native Map/Shards/S0172 00798490-0079d890|S0172]] |
| `008b5ec0` | `GetPairFromSlot3c8` |  | Third two-dword accessor in the actor object (after +0x1870 at 008b31c0 and +0x18b0 at 008b32e0); its single caller 00b5ce60 is unlabelled, so the meaning of the pair is not established. | [[Native Map/Shards/S0222 008b1e00-008b7730|S0222]] |
| `0096d370` | `UpdateActorMotionStep` |  | Per-frame actor motion step: applies the queued animation and advances movement from the queued direction and entity speed, sending jump or position updates when its conditions hold. | [[Native Map/Shards/S0258 00969a10-0096de80|S0258]] |
| `00beff30` | `ProxiedObject::SetChildFlag` |  | Updates a flag on an associated child object. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00bf04f0` | `ProxiedObject::UpdateOverheadChatBubble ⚠downgrade` | ProxiedObject | Adds or refreshes a chat bubble in the proxied object's overhead display set. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |

## 🟡 speculative (9)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00787e70` | `Actor::TimedNodeSweep ?` | Actor | Returns unless byte this+0x240 is set. Reads the 64-bit ms tick (007027f0) and compares it with the stored tick at +0x768/+0x76c. When more than 60000 ms have passed, walks the node list at +0x3dc: a node qualifies when… | [[Native Map/Shards/S0169 00787e70-0078b5f0|S0169]] |
| `00788a70` | `Actor::QueryKeyedFloat ?` | Actor | Returns 0 unless the second stack argument equals -0x4ebba5b5. Then writes one float to the out pointer (first stack argument): DAT_0175b420 minus max(this+0x258, this+0x260) when byte this+0x226 is set, otherwise DAT_0… | [[Native Map/Shards/S0169 00787e70-0078b5f0|S0169]] |
| `0079c3d0` | `ActorManagerInterfaceConstFloat ?` | ActorManagerInterface | Virtual slot [8] of ActorManagerInterface (bytes d9 05 imm32; c3): loads the float constant at 017ed7d4 onto the x87 stack and returns it. | [[Native Map/Shards/S0172 00798490-0079d890|S0172]] |
| `0079c620` | `NodePairMoveLinks ?` |  | Thiscall (ecx = destination holder, one stack argument = source node). Unlinks the source from the chain formed by its +0x1d0 and +0x1d4 links: when +0x1d0 is NULL stores +0x1d4 into the destination's +4, otherwise link… | [[Native Map/Shards/S0172 00798490-0079d890|S0172]] |
| `0079c690` | `HashChainUnlinkById1024 ?` |  | Thiscall (ecx = table holder, one stack argument = entry). Computes the bucket at table+0x10 + (entry id at +0x1cc & 0x3ff) * 4, walks the chain through the next link at +0x1c8, and unlinks the entry when found, clearin… | [[Native Map/Shards/S0172 00798490-0079d890|S0172]] |
| `0079ca50` | `ListPopHeadAndUnlink ?` |  | Fastcall (ecx = holder with head at +4 and count at +0xc). When the count is positive, takes the head entry, removes it from its id bucket through 0079c690, moves its link pair through 0079c620, decrements the count and… | [[Native Map/Shards/S0172 00798490-0079d890|S0172]] |
| `0079cdf0` | `EntryListDrainToPool ?` |  | Fastcall (ecx = holder with list head at +0x38, tail at +0x3c, count at +0x40). Loops while the count is positive: takes the head entry, unlinks it from its id bucket through 0079c690 and from the list (+0x1d0/+0x1d4 li… | [[Native Map/Shards/S0172 00798490-0079d890|S0172]] |
| `00bf01e0` | `ProxiedObject::GetTransformData ? ⚠rename` | ProxiedObject | Provides a pointer to a shared transform-related object field. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00bf0d20` | `ProxiedObject::SpawnTimedOverheadEffect ? ⚠rename` | ProxiedObject | Creates a timed overhead effect on proxied objects. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |

## ⚪ unknown (8)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00799700` | `— ⚠reject` | ActorGroupRequestor | Constructor of ActorGroupRequestor (AssetRequester base). Its virtual methods are 007994b0, 007994c0, 007997a0, 007995d0 and 0079a880 in this shard. | [[Native Map/Shards/S0172 00798490-0079d890|S0172]] |
| `008b3320` | `— ⚠reject` |  | Recorder of an interaction-acquisition radius on the actor: the vault's interaction-range note proves the AddNpc dispatch 0092f997 substitutes the session fallback when the parsed value is non-positive and stores the re… | [[Native Map/Shards/S0222 008b1e00-008b7730|S0222]] |
| `0096b520` | `— ⚠reject` |  | Despawn routine: marks the actor as despawning, releases its handles and creates fade and remove event objects; the referenced message string reports a non-positive fade time. | [[Native Map/Shards/S0258 00969a10-0096de80|S0258]] |
| `00bfe090` | `— ⚠reject` | PlayerTrackable | Initializes the PlayerTrackable base object. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00bfe170` | `— ⚠reject` |  | Copies eight dwords assembled from offsets 0x20 through 0x2c and 0x1818 into the eight-dword block beginning at this+0x1820. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00bfe1c0` | `— ⚠reject` |  | Copies eight dwords assembled from offsets 0x20 through 0x2c and 0x1818 into the eight-dword block beginning at this+0x1820. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00bfe2e0` | `— ⚠reject` | PlayerTrackable | Returns a data block, using the associated object when present and an inline fallback otherwise. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00bfe410` | `— ⚠reject` | PlayerSphereTrackable | Returns a data block, using the associated object when present and an inline fallback otherwise. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |

