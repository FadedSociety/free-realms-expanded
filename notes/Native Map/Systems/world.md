# System: world

43 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (13)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008a7840` | `SetZoneType` |  | Sets the current zone streaming type on the client object and updates the named configuration value; called by the packet router 009471a0. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008aa1b0` | `HashListMap_MovementController_Construct` | SoeUtil::HashListMap | Registers a movement controller per signature character guid - the world/actor movement bookkeeping table. | [[Native Map/Shards/S0221 008a86b0-008b1d80|S0221]] |
| `008d3460` | `SetRenderFloraFromZoneData` |  | Applies the housing/zone data's flora-rendering flag to the renderer; its caller 00ac36c0 is the accepted HousingProcessor::HandleZoneData (housing subtype 45), so this is the zone-data-to-render bridge. | [[Native Map/Shards/S0227 008d0920-008d5c60|S0227]] |
| `008ed660` | `SetWorldNameAndLoadRoadmap` |  | Sets the active world name and loads its roadmap file. | [[Native Map/Shards/S0233 008ec2a0-008f0cc0|S0233]] |
| `008ed7b0` | `SetZoneTypeFromName` |  | Maps the zone-type text from the zone data to the value stored at this+0x190. | [[Native Map/Shards/S0233 008ec2a0-008f0cc0|S0233]] |
| `0093d180` | `BeginPlayerZoning` |  | Move the local client into a destination zone while clearing stale gameplay and housing state. | [[Native Map/Shards/S0248 009392c0-0093d740|S0248]] |
| `00b0a290` | `ReadSphereCenterAndRadius` | ClientAreaDataShaped<SoeUtil::Math::Sph… | Reads sphere center/radius fields into the shaped-data object. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00cecbc0` | `StoreAndDispatchVector4At50` | ClientAreaCompositeEffectProperty | Virtual vector setter for the composite-effect property. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00cecbf0` | `StoreAndDispatchVector4At60` | ClientAreaCompositeEffectProperty | Virtual vector setter for the composite-effect property. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00cecc20` | `SetScaleAboveThresholdAndDispatch` | ClientAreaCompositeEffectProperty | Applies an accepted scale update to registered effect targets. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00cece40` | `ClientAreaCompositeEffectProperty::Update` | ClientAreaCompositeEffectProperty | Updates local effect state and conditionally builds the broadcast packet; construction is visible but packet send is not. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00ced930` | `ClientAreaMinimapProperty::Enter` | ClientAreaMinimapProperty | Minimap area-entry hook; exact semantic difference from the paired exit hook is not visible in the body. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00ced950` | `ClientAreaMinimapProperty::Exit` | ClientAreaMinimapProperty | Minimap area-exit hook; exact semantic difference from the paired entry hook is not visible in the body. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |

## 🟢 likely (19)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008e4330` | `SphereTreeFirstOverlapCandidate` |  | Provides the first overlap candidate; caller 00a2ac20 applies actor type, hostility and dead-state checks for tab targeting. | [[Native Map/Shards/S0230 008deaaa-008e47b0|S0230]] |
| `008eb490` | `WorldMeshEmitterEvent::UpdateEmitter ⚠downgrade` | WorldMeshEmitterEvent | Initializes the event's world-mesh emitter effect. | [[Native Map/Shards/S0232 008e82e0-008ec1d0|S0232]] |
| `009613d0` | `SetZoneFlagsAndResetQueue` |  | Called by BeginPlayerZoning; the caller supplies the zone transition context. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `00985340` | `CreateActorNodeIfMissing` |  | Shared actor-node creation on lookup miss. The directly cited Stage7 note establishes the AddNpc caller path, while callers 00b56dd0 and 00b57170 also use this routine for mannequin and pet-target records; the AddNpc-sp… | [[Native Map/Shards/S0263 00982f10-009897a0|S0263]] |
| `00985550` | `RemoveActorNodeAndMaybeUnregisterGuidPair` |  | Manager-side actor-node cleanup with conditional GUID-pair unregistration. The condition is bit 1 of node byte +0x141; this body does not establish that bit?s meaning. AddNpc/deletion call relationships are caller conte… | [[Native Map/Shards/S0263 00982f10-009897a0|S0263]] |
| `009855d0` | `FindNodeByKeyPairAndDestroy` |  | Key-pair lookup/deletion wrapper. The PetsHandler::ResetTrackedPair caller is contextual evidence for one use; the wrapper itself does not name the key domain. | [[Native Map/Shards/S0263 00982f10-009897a0|S0263]] |
| `00985600` | `RebuildIndexesAroundParam2` |  | Rebuilds the indexed lists around param_2 and conditionally updates a GUID-pair registry. The interpretation as a tracked/selected node follows the pointer at the nested +0xc8 field and callers; the class name is not es… | [[Native Map/Shards/S0263 00982f10-009897a0|S0263]] |
| `009856d0` | `CleanupNodeIndexesAndOwnedObject` |  | Clears manager-associated node indexes and an owned object. The manager ownership interpretation follows caller 008a5e80 and the related constructor 00984fe0; this body has no owning-class RTTI name. | [[Native Map/Shards/S0263 00982f10-009897a0|S0263]] |
| `00a3bca0` | `SetMember24c8FromGlobalService` |  | Setter that rebuilds the object referenced by member +0x24c8 from three arguments plus a constant 16-byte global (the 1.0 float suggests a unit/scale argument); the paired null-installer is 00a3bd50 and both act on the … | [[Native Map/Shards/S0295 00a39b00-00a3f350|S0295]] |
| `00b09d90` | `GetAreaShapeData` | ClientAreaDataShaped<SoeUtil::Math::Sph… | Returns the shaped-data subobject at offset 0x1e0; exact member semantics are not established. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00b09db0` | `GetAreaShapeData` | ClientAreaDataShaped<ClientAreaDome> | Returns the shaped-data subobject at offset 0x1e0; exact member semantics are not established. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00b09dd0` | `GetAreaShapeData` | ClientAreaDataShaped<SoeUtil::Math::Box… | Returns the shaped-data subobject at offset 0x1e0; exact member semantics are not established. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00b09e10` | `GetAreaShapeData` | ClientAreaDataShaped<ClientAreaDome> | Returns the shaped-data subobject at offset 0x1e0; exact member semantics are not established. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00b09e20` | `GetAreaShapeName` | ClientAreaDataShaped<SoeUtil::Math::Sph… | Returns the literal shape-name string. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00b09e30` | `GetAreaShapeName` | ClientAreaDataShaped<SoeUtil::Math::Box… | Returns the static shape-name data pointer; the pointed-to bytes identify box. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00b09e90` | `WriteFloorShapeTag` | ClientAreaFloor | Writes the shape=floor tag pair through the shared data writer. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00b0a160` | `Method_00b0a160 → DrawSphereAreaDebugShape` | ClientAreaDataShaped<SoeUtil::Math::Sph… | Shape class is from RTTI; routine sends stored values and selected color to the renderer. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00b0a1a0` | `Method_00b0a1a0 → DrawBoxAreaDebugShape` | ClientAreaDataShaped<SoeUtil::Math::Box… | Box class is from RTTI; routine copies stored bounds and static parameters before drawing. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00b0a250` | `Method_00b0a250 → DrawDomeAreaDebugShape` | ClientAreaDataShaped<ClientAreaDome> | Dome class is from RTTI; routine sends stored values and selected color to the renderer. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |

## 🟡 speculative (9)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0090e8a0` | `PlayClient::SetStartingZoneState ?` | PlayClient | Updates the client starting-zone state and transition hooks. | [[Native Map/Shards/S0239 0090a890-009105b0|S0239]] |
| `009584b0` | `SetTransformAndNotify ?` |  | Calls virtual slot 7, copies four transform words through 00beff10, then invokes 00995e10 when linked context field +0x5e8 is nonzero. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `00b09e40` | `ReturnStaticShapeNameData ? → GetAreaShapeName` | ClientAreaDataShaped<ClientAreaDome> | Returns the address of static data at 0181d274; its contents are not proven by this function body. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00b09e50` | `ForwardAreaDataBoolResult ?` | ClientAreaDataShaped<SoeUtil::Math::Sph… | Calls the shared area-data validation helper and returns whether validation succeeded. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00b09e70` | `ForwardAreaDataBoolResult ?` | ClientAreaDataShaped<SoeUtil::Math::Box… | Calls the shared area-data validation helper and returns whether validation succeeded. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00b09eb0` | `SetReadyFlagWhenArgumentZero ? ⚠rename` |  | Shared virtual implementation reported in both ClientAreaDataLoader and ClientAreaDataManager vtables. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00b09ec0` | `ClearLoaderStateFields ?` | ClientAreaDataLoader | Clears two loader state fields; their meanings are not established. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00b0a100` | `TestDomeBounds ? ⚠rename` | ClientAreaDataShaped<ClientAreaDome> | Shape class is from RTTI; detailed geometry semantics depend on the called helper. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00cec490` | `TransformAndDispatchCompositeEffect ?` |  | Transform semantics are inferred from matrix operations and target callbacks; target class is unknown. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |

## ⚪ unknown (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008ff3d0` | `— ⚠reject` |  | Reads an id from object this+0xc8 via vcall slot 7 (+0x1c) and FUN_00790460, and may replace it through FUN_010ba6e0 and the +0x20/+0x1c/+0x2b0/+0xa0 chain. When flag this+0x894 is set and the 64-bit time from FUN_00702… | [[Native Map/Shards/S0237 008ff3d0-00904e70|S0237]] |
| `00984fe0` | `— ⚠reject` |  | Initializes node indexes and fields for an object used by the client actor-node routines. Its enclosing class name is not established by RTTI; caller 0092b170 is PlayClient initialization, and the node-manager interpret… | [[Native Map/Shards/S0263 00982f10-009897a0|S0263]] |

