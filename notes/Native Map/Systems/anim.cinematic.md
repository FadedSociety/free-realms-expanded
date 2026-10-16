# System: anim.cinematic

25 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a6f850` | `_anon_34A1D9ED::ControllerCinematicActorHandler::ReleaseNameString` | _anon_34A1D9ED::ControllerCinematicActo… | Member teardown for the cinematic actor handler's text member at +4. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6f8d0` | `_anon_34A1D9ED::ControllerCinematicActorHandler::ControllerCinematicActorHandler` | _anon_34A1D9ED::ControllerCinematicActo… | Constructs the cinematic actor handler (derived from ActorHandler). | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6f940` | `_anon_34A1D9ED::ControllerCinematicActorHandler::ScalarDeletingDtor` | _anon_34A1D9ED::ControllerCinematicActo… | Deleting-destructor entry (vtable slot 0) of the cinematic actor handler. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00d009c0` | `GetTimelineEntryField4ByIndex` |  | Supplies a timeline entry value to the cinematic caller. | [[Native Map/Shards/S0426 00cfc4e0-00d01c70|S0426]] |

## 🟢 likely (9)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a6f2e0` | `ControllerCinematic::BeginTimedInterpolation` | ControllerCinematic | Starts timed (state 1) interpolation of the cinematic controller's transform over a duration. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6f3c0` | `ControllerCinematic::BeginStateOneWithRefs` | ControllerCinematic | Enters timed state 1 with two referenced objects and stores the blend value at +0x110 used by the state-1 update. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6f520` | `ControllerCinematic::ScalarDeletingDtor` | ControllerCinematic | Deleting-destructor entry of the cinematic controller in its secondary vtable. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6f740` | `ReleaseGlobalCinematicHandle` |  | Shutdown-time release of the global cinematic handle, called from BaseClient::Shutdown (009359e0). | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6fa90` | `ControllerCinematic::StartPlaybackState2` | ControllerCinematic | Starts cinematic playback state 2 for an actor (called from 008c6290). | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6fc20` | `ControllerCinematic::StartPlaybackState3` | ControllerCinematic | Starts cinematic playback state 3 for an actor with a second referenced object (called from 008c6350). | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6fdb0` | `ControllerCinematic::SetFlag40IfState23` | ControllerCinematic | Turns on the actor-active flag 0x40 for states 2 and 3; reached through the type-gated caller 008a6d80. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6fe20` | `ControllerCinematic::ClearFlag40IfState23` | ControllerCinematic | Turns off the actor-active flag 0x40 for states 2 and 3; reached through the type-gated caller 008a6db0. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a70790` | `ControllerCinematic::SetCinematicModeFlag` | ControllerCinematic | Switches the cinematic-mode flag and announces GameEvents:OnCinematicModeOn when it turns on. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |

## 🟡 speculative (8)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a6f270` | `ControllerCinematic::BindManagerObjectAndNotify ? ⚠rename` | ControllerCinematic | Virtual in the secondary vtable at 01815b9c, slot 7. Calls 0094e210, 007935a0 (with _DAT_01be7018) and 0094f3c0; clears a byte in the sub-object at (this+0x74)+0x1a8 +0x88; gets the manager through 0094cf60 and, when it… | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6f400` | `ControllerCinematic::ForwardField2cToSlot1 ?` | ControllerCinematic | Virtual in the secondary vtable at 01815b9c, slot 48 (+0xc0), 31 bytes: when the dword at +0x2c is nonzero and the object at +0x11c is non-null, it replaces the first argument with the +0x2c value and jumps to that obje… | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6f540` | `_anon_34A1D9ED::ControllerCinematicActorHandler::ApplyTimeAndNotifyEntries ?` | _anon_34A1D9ED::ControllerCinematicActo… | Virtual slot 1 of the cinematic actor handler. Clamps the float returned by 007707f0 to at least the constant DAT_01815b84 and stores it at +0x98 of the object at param_2+0x2b0 when that pointer is non-null. Sets the gl… | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6f770` | `_anon_34A1D9ED::ControllerCinematicActorHandler::ReleaseGlobalHandleAndClear ? …` | _anon_34A1D9ED::ControllerCinematicActo… | Releases and clears the global cinematic handle when the actor handler is torn down. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6f7b0` | `_anon_34A1D9ED::ControllerCinematicActorHandler::ClearGlobalHandleIfEqual ?` | _anon_34A1D9ED::ControllerCinematicActo… | Virtual slot 13 (+0x34), 24 bytes: when the global DAT_01be701c equals param_1 it sets the global to 0; otherwise does nothing. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6f7d0` | `CinematicHandle_QueryActorDeltaPositive ?` |  | Asks the manager (00769710, then its +0xf0 object slot 4 with the global handle DAT_01be701c) whether the handle is valid. If so, calls 00784c80 on the field at this+0x120 and follows the returned object's +0x14 and +0x… | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6fa10` | `ControllerCinematic::ReleaseDetachedActor ?` | ControllerCinematic | Virtual in the secondary vtable at 01815b9c, slot 32 (+0x80), 115 bytes: when the state at +0xe8 is 2 or 3, bit 0x80 of +0xec is set and the object at +0x11c is non-null, and the handle (manager +0xf0 object slot 0x10 w… | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a707f0` | `CinematicBoneModel_LookupAndBind ? ⚠rename` |  | Resolves the Cinematic.BoneName and Cinematic.ModelId settings from a keyed table and records a manager handle for the cinematic system. | [[Native Map/Shards/S0305 00a707f0-00a75b20|S0305]] |

## ⚪ unknown (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a6f1c0` | `— ⚠reject` | ControllerCinematic | Non-deleting destructor body of the cinematic controller, called from its deleting entry 00a6f520. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6f960` | `— ⚠reject` | ControllerCinematic | Leaves cinematic mode: resets the controller state and fires the OnCinematicModeOff event. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6fe90` | `— ⚠reject` | ControllerCinematic | Per-frame update of the cinematic controller: timed interpolation for state 1, a follow path for state 2, and a blended path for state 3. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a706a0` | `— ⚠reject` | ControllerCinematic | Constructs the cinematic controller with two embedded Target sub-objects. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |

