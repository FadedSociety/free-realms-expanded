# System: game.movement

35 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (18)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008b3180` | `GetEntitySpeed` |  | Entity speed getter with a reduced-speed variant: the vault's NPC motion research records that the state-bit-0 call site passes 0.0 as the speed parameter while the clear path calls 008b3180, so the returned value is th… | [[Native Map/Shards/S0222 008b1e00-008b7730|S0222]] |
| `008bb440` | `InitiateMountControllerHandler::ResetController` | InitiateMountControllerHandler | Resets movement/controller flags for this handler. | [[Native Map/Shards/S0223 008b7790-008bcf40|S0223]] |
| `0095dae0` | `SetHorizontalDirection` |  | Applies a nonzero horizontal direction to an entity and its model. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `00bf9430` | `UpdateMovementState` |  | Per-frame movement/state update called by UpdateMotionAndActorState; one branch sends a safe-location request. Its trigger condition remains unresolved. | [[Native Map/Shards/S0381 00bf9430-00bfcf30|S0381]] |
| `00bfba00` | `ProxiedCharacterMovementController::~ProxiedCharacterMovementController` | ProxiedCharacterMovementController | Base movement-controller destructor body used by derived-controller cleanup. | [[Native Map/Shards/S0381 00bf9430-00bfcf30|S0381]] |
| `00bfba80` | `ProxiedCharacterMovementController::ScalarDeletingDestructor` | ProxiedCharacterMovementController | Implements conditional deletion for the base movement controller. | [[Native Map/Shards/S0381 00bf9430-00bfcf30|S0381]] |
| `00bfbaa0` | `ProxiedCharacterRailMovementController::AdvanceRail` | ProxiedCharacterRailMovementController | Advances a proxied character along one rail path and applies the resulting transform. | [[Native Map/Shards/S0381 00bf9430-00bfcf30|S0381]] |
| `00bfbd70` | `ProxiedCharacterRailMovementController::~ProxiedCharacterRailMovementController` | ProxiedCharacterRailMovementController | Releases the rail controller's owned route state before base-controller cleanup. | [[Native Map/Shards/S0381 00bf9430-00bfcf30|S0381]] |
| `00bfbde0` | `ProxiedCharacterRailMovementController::ScalarDeletingDestructor` | ProxiedCharacterRailMovementController | Implements conditional deletion for the rail movement controller. | [[Native Map/Shards/S0381 00bf9430-00bfcf30|S0381]] |
| `00bfbe10` | `ProxiedCharacterRelativeRailMovementController::AdvanceRelativeRail` | ProxiedCharacterRelativeRailMovementCon… | Moves a proxied character along paired relative rails and applies the resulting transform. | [[Native Map/Shards/S0381 00bf9430-00bfcf30|S0381]] |
| `00bfbfb0` | `ProxiedCharacterRelativeRailMovementController::ProxiedCharacterRelativeRailMov…` | ProxiedCharacterRelativeRailMovementCon… | Creates a proxied character controller for paired relative-rail movement. | [[Native Map/Shards/S0381 00bf9430-00bfcf30|S0381]] |
| `00bfc0e0` | `ProxiedCharacterRelativeRailMovementController::~ProxiedCharacterRelativeRailMo…` | ProxiedCharacterRelativeRailMovementCon… | Releases the two route states owned by the relative-rail controller. | [[Native Map/Shards/S0381 00bf9430-00bfcf30|S0381]] |
| `00bfc170` | `ProxiedCharacterRelativeRailMovementController::ScalarDeletingDestructor` | ProxiedCharacterRelativeRailMovementCon… | Implements conditional deletion for the relative-rail movement controller. | [[Native Map/Shards/S0381 00bf9430-00bfcf30|S0381]] |
| `00bfc190` | `ProxiedCharacterSeekMovementController::ResetTargetData` | ProxiedCharacterSeekMovementController | Resets or replaces the seek controller's embedded Target state through its slot-2 operation. | [[Native Map/Shards/S0381 00bf9430-00bfcf30|S0381]] |
| `00bfc280` | `ProxiedCharacterSeekMovementController::ProxiedCharacterSeekMovementController` | ProxiedCharacterSeekMovementController | Creates a seek controller with an initialized Target subobject and starting orientation. | [[Native Map/Shards/S0381 00bf9430-00bfcf30|S0381]] |
| `00bfc440` | `ProxiedCharacterSeekMovementController::ScalarDeletingDestructor` | ProxiedCharacterSeekMovementController | Implements conditional deletion for the seek movement controller. | [[Native Map/Shards/S0381 00bf9430-00bfcf30|S0381]] |
| `00bfc460` | `ProxiedCharacterSeekMovementController::Update` | ProxiedCharacterSeekMovementController | Advances a proxied character toward the controller's Target while smoothing motion and orientation. | [[Native Map/Shards/S0381 00bf9430-00bfcf30|S0381]] |
| `00cc4410` | `ClientInterpolatedMovementController::LookupTransformForKeyPair` | ClientInterpolatedMovementController | Returns the transform-like four-word value associated with the supplied key pair, or the initialized default when no node exists. | [[Native Map/Shards/S0415 00cc1a00-00cc6bc0|S0415]] |

## 🟢 likely (7)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008c5a10` | `PositionDeltaGate_Check` |  | Mechanical distance-threshold gate on a stored four-float position; the position-update senders (SendJumpUpdate 0092e670 and others, accepted) call it, which is the only evidence for a movement/position role. | [[Native Map/Shards/S0225 008c52a0-008ca980|S0225]] |
| `009607c0` | `UpdateActionAndRelatedState` |  | Mixed character-state update; the dismount helper and fields suggest movement or mount state, but the exact state name is unresolved. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `00963500` | `SetPositionUpdateStateFlag` |  | A local state transition invokes the confirmed PlayerUpdatePacketUpdatePosition sender when the requested flag is true. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `009635f0` | `ClearModeAndSendPositionUpdate` |  | The called helper sends a position update after the mode bit changes. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `00963ad0` | `ComputeMovementTowardTarget` |  | Combines movement-state changes with position/jump update send helpers; the exact timer and speed constants remain unnamed here. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `00bfba20` | `ResolveVector4OrDefault` |  | The rail update calls this helper to obtain a four-component value; its domain meaning is unknown. | [[Native Map/Shards/S0381 00bf9430-00bfcf30|S0381]] |
| `00bfc3e0` | `ProxiedCharacterSeekMovementController::~ProxiedCharacterSeekMovementController` | ProxiedCharacterSeekMovementController | Cleans the seek controller's embedded Target subobject before its deleting wrapper frees the object. | [[Native Map/Shards/S0381 00bf9430-00bfcf30|S0381]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0095f700` | `ComputeScaledSpeedFactor ?` |  | Used by motion-processing caller 0096d370; the returned scale?s broader meaning is tentative. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |

## ⚪ unknown (9)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008bb3a0` | `— ⚠reject` | InitiateMountControllerHandler | Processes an initiate-mount controller event. | [[Native Map/Shards/S0223 008b7790-008bcf40|S0223]] |
| `0095d930` | `— ⚠reject` |  | Sets orientation from a scalar angle; the interpretation follows the transform and rotation setter calls. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `00963550` | `— ⚠reject` |  | The called helper sends a position update with mode-dependent flags. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `00963670` | `— ⚠reject` |  | The helper call sends a position update with flag value 0x82. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `00963700` | `— ⚠reject` |  | The helper call sends a position update with flag value 0x42. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `009637d0` | `— ⚠reject` |  | The helper call sends a jump update on the eligible owner transition. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `00a89bc0` | `— ⚠reject` |  | Drives the click-to-move destination arrow each frame: shows and orients it toward the chosen target and hides it when the movement mode does not apply. | [[Native Map/Shards/S0309 00a87ba0-00a8cdc0|S0309]] |
| `00bfbca0` | `— ⚠reject` | ProxiedCharacterRailMovementController | Creates a proxied character rail controller and optionally primes its first route update. | [[Native Map/Shards/S0381 00bf9430-00bfcf30|S0381]] |
| `00cc4600` | `— ⚠reject` | ClientInterpolatedMovementController | Processes a transform sample for the client interpolated movement controller and applies its collision-adjusted state. | [[Native Map/Shards/S0415 00cc1a00-00cc6bc0|S0415]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00bf9430` | `UpdateMovementState` | send | PacketZoneSafeTeleportRequest |  | Hypothesis: the original server treated this empty opcode-122 request as a safe relocation request and returned a 38/12 location update with teleport enabled. This depends on the native request class and two community emulator handlers; no original server source or capture confirms that response. |
