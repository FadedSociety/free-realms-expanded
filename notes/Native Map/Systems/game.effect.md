# System: game.effect

94 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (46)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008c51a0` | `WorldCompositeEffectEvent::Constructor` | WorldCompositeEffectEvent | Constructs a composite world effect event with its secondary interface and supplied event data. | [[Native Map/Shards/S0224 008bcf60-008c5290|S0224]] |
| `008c5240` | `WorldCompositeEffectEvent::GetResult` | WorldCompositeEffectEvent | Retrieves the delegated result from the composite event interface. | [[Native Map/Shards/S0224 008bcf60-008c5290|S0224]] |
| `008c5270` | `WorldCompositeEffectEvent::HasUnsetValue` | WorldCompositeEffectEvent | Tests the delegated event value for the unset sentinel. | [[Native Map/Shards/S0224 008bcf60-008c5290|S0224]] |
| `008c5290` | `WorldCompositeEffectEvent::AdjustorThunk` | WorldCompositeEffectEvent | Adjusts the secondary-interface this pointer before transferring control to the shared implementation. | [[Native Map/Shards/S0224 008bcf60-008c5290|S0224]] |
| `0095c230` | `PlayerEffectEvent::PlayerEffectEvent` | PlayerEffectEvent | Initializes the shared base state for player effect events. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095c370` | `PlayerEffectEvent::Process` | PlayerEffectEvent | Runs the common lifecycle logic used by PlayerEffectEvent and its particle, sound, model-material, light, camera, rumble, mesh, lightsaber-trail, and projectile subclasses. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095c530` | `PlayerParticleEmitterEvent::PlayerParticleEmitterEvent` | PlayerParticleEmitterEvent | Initializes a particle-emitter player effect event. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095c5f0` | `PlayerParticleEmitterEvent::~PlayerParticleEmitterEvent` | PlayerParticleEmitterEvent | Releases particle-emitter event ownership and base cleanup state. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095c670` | `PlayerParticleEmitterEvent::ScalarDeletingDtor` | PlayerParticleEmitterEvent | Implements the scalar deleting destructor. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095c6a0` | `PlayerSoundEmitterEvent::PlayerSoundEmitterEvent` | PlayerSoundEmitterEvent | Initializes a sound-emitter player effect event. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095c750` | `PlayerSoundEmitterEvent::~PlayerSoundEmitterEvent` | PlayerSoundEmitterEvent | Releases sound-emitter event ownership and cleanup state. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095c7d0` | `PlayerSoundEmitterEvent::ScalarDeletingDtor` | PlayerSoundEmitterEvent | Implements the scalar deleting destructor. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095c7f0` | `PlayerSoundEmitterEvent::ScalarDeletingDtorAdjustorThunk` | PlayerSoundEmitterEvent | Adjusts the secondary-vtable this pointer before entering the scalar deleting destructor. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095c8b0` | `PlayerModelMaterialEvent::~PlayerModelMaterialEvent` | PlayerModelMaterialEvent | Releases the event's referenced object and cleanup state. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095c930` | `PlayerModelMaterialEvent::ScalarDeletingDtor` | PlayerModelMaterialEvent | Implements the class's scalar deleting destructor. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095c950` | `PlayerModelMaterialEvent::ScalarDeletingDtorAdjustorThunk` | PlayerModelMaterialEvent | Adjusts the secondary-vtable this pointer before entering the scalar deleting destructor. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095c960` | `PlayerLightEmitterEvent::PlayerLightEmitterEvent` | PlayerLightEmitterEvent | Initializes a light-emitter player effect event. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095ca10` | `PlayerLightEmitterEvent::~PlayerLightEmitterEvent` | PlayerLightEmitterEvent | Releases the referenced object and event cleanup state. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095ca90` | `PlayerLightEmitterEvent::ScalarDeletingDtor` | PlayerLightEmitterEvent | Implements this class's scalar deleting destructor. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095cac0` | `PlayerCameraEffectEvent::PlayerCameraEffectEvent` | PlayerCameraEffectEvent | Initializes a camera-effect player event. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095cb70` | `PlayerCameraEffectEvent::~PlayerCameraEffectEvent` | PlayerCameraEffectEvent | Releases camera-effect event ownership and cleanup state. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095cbf0` | `PlayerCameraEffectEvent::ScalarDeletingDtor` | PlayerCameraEffectEvent | Implements the class scalar deleting destructor. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095ccd0` | `PlayerRumbleEffectEvent::~PlayerRumbleEffectEvent` | PlayerRumbleEffectEvent | Releases rumble-event ownership and cleanup state. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095cd50` | `PlayerRumbleEffectEvent::ScalarDeletingDtor` | PlayerRumbleEffectEvent | Implements the scalar deleting destructor. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095cd70` | `PlayerRumbleEffectEvent::ScalarDeletingDtorThunk` | PlayerRumbleEffectEvent | Slot-zero thunk to the scalar deleting destructor. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095ce30` | `PlayerMeshEmitterEvent::~PlayerMeshEmitterEvent` | PlayerMeshEmitterEvent | Releases the mesh-emitter event's reference and cleanup state. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095ceb0` | `PlayerMeshEmitterEvent::ScalarDeletingDtor` | PlayerMeshEmitterEvent | Implements the mesh-emitter scalar deleting destructor. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095ced0` | `PlayerMeshEmitterEvent::ScalarDeletingDtorThunk` | PlayerMeshEmitterEvent | Slot-zero thunk to the scalar deleting destructor. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095cf90` | `PlayerLightsaberTrailEvent::~PlayerLightsaberTrailEvent` | PlayerLightsaberTrailEvent | Releases the event reference and cleanup state. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095d010` | `PlayerLightsaberTrailEvent::ScalarDeletingDtor` | PlayerLightsaberTrailEvent | Implements the scalar deleting destructor. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095d030` | `PlayerLightsaberTrailEvent::ScalarDeletingDtorThunk` | PlayerLightsaberTrailEvent | Slot-zero thunk to the scalar deleting destructor. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `00961420` | `SetFlyByEffectId` |  | Validates and stores a fly-by effect definition identifier. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `00964e00` | `PlayerProjectileEvent::Constructor` | PlayerProjectileEvent | Constructs a projectile-specific PlayerEffectEvent with an owned Target array. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `00964f20` | `PlayerProjectileEvent::Destructor` | PlayerProjectileEvent | Destructor body for the projectile event and its owned members. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `00964fd0` | `PlayerProjectileEvent::ScalarDeletingDestructor` | PlayerProjectileEvent | Scalar-deleting destructor for PlayerProjectileEvent. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `00964ff0` | `ActorProjectileEmitter::Constructor` | ActorProjectileEmitter | Constructs an actor projectile emitter with string, projectile-parameter and target-array members. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `009650e0` | `ActorProjectileEmitter::IsSlot0cClear` | ActorProjectileEmitter | Virtual slot 4 exposes the negation of the slot +0xc result. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `00965100` | `ActorProjectileEmitter::ScalarDeletingDestructor` | ActorProjectileEmitter | Scalar-deleting destructor for ActorProjectileEmitter. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `00965120` | `PlayerProjectileEvent::CreateActorProjectileEmitter` | PlayerProjectileEvent | Virtual slot 9 creates a projectile emitter from the event's context when the required object is available. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `00a13690` | `ClientEffectTag_ScalarDeletingDtor` | ClientEffectTag | Scalar-deleting path for a ClientEffectTag; the call runs the EffectTag base destructor before the optional heap free. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00a162b0` | `HashListMap_ClientEffectTag_20_minus1_Dtor` | SoeUtil::HashListMap<int,ClientEffectTa… | In-place destruction of the per-client effect-tag map; the same map is erased one node at a time by 00a15440 when a power hour ends. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00a16410` | `HashListMap_ClientEffectTag_20_minus1_ScalarDeletingDtor` | SoeUtil::HashListMap<int,ClientEffectTa… | Heap-delete path of the effect-tag map. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00a170f0` | `HashListMap_ClientEffectTag_20_8_Dtor` | SoeUtil::HashListMap<int,ClientEffectTa… | Full teardown of the effect-tag map variant that carries an allocator sub-object, run from 00a187a0/00a19240/00a19680/00a1d6d0 and four EH scopes. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00a17b70` | `HashListMap_ClientEffectTag_20_8_AllocatorSlot1` | SoeUtil::HashListMap<int,ClientEffectTa… | Vtable slot 1 of the effect-tag map, i.e. the allocate hook used when the map needs a node. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00a17b80` | `HashListMap_ClientEffectTag_20_8_AllocatorSlot2` | SoeUtil::HashListMap<int,ClientEffectTa… | Vtable slot 2 of the effect-tag map, i.e. the deallocate hook used when a node is freed. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00a187a0` | `HashListMap_ClientEffectTag_20_8_ScalarDeletingDtor` | SoeUtil::HashListMap<int,ClientEffectTa… | Heap-delete path of the effect-tag map variant with the embedded pool. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |

## 🟢 likely (25)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008bf990` | `EffectTag::ScalarDeletingDestructor` | EffectTag | Runs the EffectTag non-deleting destructor, then conditionally frees the object. | [[Native Map/Shards/S0224 008bcf60-008c5290|S0224]] |
| `0095c660` | `PlayerParticleEmitterEvent::GetWordAt50` | PlayerParticleEmitterEvent | Reads the field at +0x50; its semantic role is not identified here. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095c7c0` | `PlayerSoundEmitterEvent::GetWordAt50` | PlayerSoundEmitterEvent | Reads the field at +0x50; its semantic role is not identified here. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095c800` | `PlayerModelMaterialEvent::PlayerModelMaterialEvent` | PlayerModelMaterialEvent | Initializes a PlayerModelMaterialEvent and retains its referenced object; the purpose of the zero-count virtual callback is unknown. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095c920` | `PlayerModelMaterialEvent::GetWordAt50` | PlayerModelMaterialEvent | Reads the field at +0x50; its semantic role is not identified here. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095ca80` | `PlayerLightEmitterEvent::GetWordAt50` | PlayerLightEmitterEvent | Reads the field at +0x50; its semantic role is not identified here. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095cbe0` | `PlayerCameraEffectEvent::GetWordAt50` | PlayerCameraEffectEvent | Reads the field at +0x50; its semantic role is not identified here. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095cc20` | `PlayerRumbleEffectEvent::PlayerRumbleEffectEvent ⚠downgrade` | PlayerRumbleEffectEvent | Initializes a rumble-effect player event. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095cd40` | `PlayerRumbleEffectEvent::GetWordAt50` | PlayerRumbleEffectEvent | Reads the field at +0x50; its semantic role is not identified here. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095cd80` | `PlayerMeshEmitterEvent::PlayerMeshEmitterEvent` | PlayerMeshEmitterEvent | Initializes a PlayerMeshEmitterEvent and retains its referenced object; the purpose of the zero-count virtual callback is unknown. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095cea0` | `PlayerMeshEmitterEvent::GetWordAt50` | PlayerMeshEmitterEvent | Reads the field at +0x50; its semantic role is not identified here. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095cee0` | `PlayerLightsaberTrailEvent::PlayerLightsaberTrailEvent ⚠downgrade` | PlayerLightsaberTrailEvent | Initializes a lightsaber trail player event. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095d000` | `PlayerLightsaberTrailEvent::GetWordAt50` | PlayerLightsaberTrailEvent | Reads the field at +0x50; its semantic role is not identified here. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `00962dd0` | `InitializeSensoryEffectRecordFields` |  | Initializes a record with an embedded sensory-effect-definition array; no owning RTTI class is visible here. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `00a0fcd0` | `ClientEffectTag_Ctor` | ClientEffectTag | Creates the client-side effect tag object used by the HashListMap<int,ClientEffectTag,20> effect registry. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a0fd30` | `ClientEffectTag_Dtor` | ClientEffectTag | Destroys a ClientEffectTag instance; also used from the EH unwind funclets 015ada30/015ada60/015adbe0/015adc70. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a0fd80` | `ClientEffectTag_ScalarDeletingDtor` | ClientEffectTag | MSVC deleting destructor for ClientEffectTag. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a10f50` | `ClientEffectTag_Dtor2` | ClientEffectTag | Destructor variant of the ClientEffectTag class (same shape as 00a0fd30). | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a10fa0` | `ClientEffectTagDerived_Ctor` | ClientEffectTag | Constructor for a derived effect tag that adds a 4-byte payload at +0x80. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a15a10` | `HashListMap_ClientEffectTag_Clear` |  | Clear-all of the per-client effect-tag map, invoked from the map destructors in this shard. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00a166c0` | `HashListMap_ClientEffectTag_InsertAtTail` | SoeUtil::HashListMap<int,ClientEffectTa… | Tail-append primitive of the effect-tag map whose removal counterpart is 00a14750; the caller 00a18a10 fills the key at +0x80 before calling. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00a18840` | `HashListMap_ClientEffectTag_20_8_Ctor ⚠downgrade` | SoeUtil::HashListMap<int,ClientEffectTa… | Builds the effect-tag map with its embedded node pool, used by the client for timed effect tags such as the Power Hour. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00a18a10` | `ClientEffectMap_CreateEntry` |  | Creation path of the per-client effect-tag map, used by 00a19e70 when a new tag arrives. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00a19e70` | `FindOrCreateBucketRecordAndDecode` |  | Search/insertion helper for a 20-bucket client record map. Its effect-tag association is inferred from the entry constructor/decoder and enclosing client state, not named in this body. | [[Native Map/Shards/S0289 00a18a90-00a1dd30|S0289]] |
| `00a19f00` | `InitBoundedReaderAndForwardKey` |  | Reader setup wrapper called by BaseClient::HandleClientUpdate at 009392c0; the client-update association comes from that caller. | [[Native Map/Shards/S0289 00a18a90-00a1dd30|S0289]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009644f0` | `InitializeSensoryEffectRecordWrapper ?` |  | Exception-cleanup wrapper around the sensory-effect record initializer. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |

## ⚪ unknown (22)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0095c690` | `— ⚠reject` | PlayerParticleEmitterEvent | Vtable thunk for the scalar deleting destructor. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095cab0` | `— ⚠reject` | PlayerLightEmitterEvent | Slot-zero thunk to the scalar deleting destructor. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095cc10` | `— ⚠reject` | PlayerCameraEffectEvent | Slot-zero thunk to the scalar deleting destructor. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095e950` | `— ⚠reject` |  | Likely creates a PlayerParticleEmitterEvent event; that class interpretation is based on the constructor this routine calls. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095ea60` | `— ⚠reject` |  | Likely creates a PlayerSoundEmitterEvent event; that class interpretation is based on the constructor this routine calls. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095eb50` | `— ⚠reject` |  | Likely creates a PlayerModelMaterialEvent event; that class interpretation is based on the constructor this routine calls. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095ec40` | `— ⚠reject` |  | Likely creates a PlayerLightEmitterEvent event; that class interpretation is based on the constructor this routine calls. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095ed30` | `— ⚠reject` |  | Likely creates a PlayerCameraEffectEvent event; that class interpretation is based on the constructor this routine calls. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095ee20` | `— ⚠reject` |  | Likely creates a PlayerRumbleEffectEvent event; that class interpretation is based on the constructor this routine calls. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095ef10` | `— ⚠reject` |  | Likely creates a PlayerMeshEmitterEvent event; that class interpretation is based on the constructor this routine calls. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095f000` | `— ⚠reject` |  | Likely creates a PlayerLightsaberTrailEvent event; that class interpretation is based on the constructor this routine calls. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095f760` | `— ⚠reject` | PlayerParticleEmitterEvent | Returns the result of the event-specific helper for the resolved player/effect resource; the exact property represented by that value is unknown. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095f860` | `— ⚠reject` | PlayerSoundEmitterEvent | Returns the result of the event-specific helper for the resolved player/effect resource; the exact property represented by that value is unknown. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095f950` | `— ⚠reject` | PlayerModelMaterialEvent | Returns the result of the event-specific helper for the resolved player/effect resource; the exact property represented by that value is unknown. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095fa30` | `— ⚠reject` | PlayerLightEmitterEvent | Returns the result of the event-specific helper for the resolved player/effect resource; the exact property represented by that value is unknown. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095fb20` | `— ⚠reject` | PlayerCameraEffectEvent | Returns the result of the event-specific helper for the resolved player/effect resource; the exact property represented by that value is unknown. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095fc10` | `— ⚠reject` | PlayerRumbleEffectEvent | Returns the result of the event-specific helper for the resolved player/effect resource; the exact property represented by that value is unknown. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095fcf0` | `— ⚠reject` | PlayerMeshEmitterEvent | Returns the result of the event-specific helper for the resolved player/effect resource; the exact property represented by that value is unknown. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095fdd0` | `— ⚠reject` | PlayerLightsaberTrailEvent | Returns the result of the event-specific helper for the resolved player/effect resource; the exact property represented by that value is unknown. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `00960af0` | `— ⚠reject` |  | The switch includes projectile-like event constructors and ActorRumbleEffectEvent; exact type names for the other cases are not recovered. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `00a14750` | `— ⚠reject` | SoeUtil::HashListMap<int,ClientEffectTa… | Erase/delete primitive of the per-client effect-tag map; called by 00a15440 when a power-hour tag ends and by the map teardown paths 00a162b0 and 00a15a10. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00a15440` | `— ⚠reject` |  | Ends a player's Power Hour effect tag: the wiki documents Power Hour as the membership ability that doubles coin earnings in the first hour of a day, so this routine removes the tag record and tells the HUD through one … | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00a15440` | `— ⚠reject` | recv | ClientUpdatePacketRemoveEffectTag |  | Hypothesis: a peer could send the 38/17 remove-effect event that reaches caller 009392c0 case 0x11; this client then removes the keyed ClientEffectTag and stops the Power Hour HUD when applicable. This depends on header reader 008d81b0, that dispatcher case, and the corresponding 38/16 add path; the catalogue supports wire shape, not original-server behavior. |
