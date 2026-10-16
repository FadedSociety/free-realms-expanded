# System: game.fishing

41 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (7)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b69860` | `FishingProcessor::FishingProcessor` | FishingProcessor | Initializes the native fishing processor and its local data sources. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b69d40` | `FishingProcessor::Cleanup → FishingProcessor::~FishingProcessor` | FishingProcessor | Destroys FishingProcessor-owned state and releases its references. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b6a6c0` | `ClearAndDeleteFishingProcessor` | FishingProcessor | Scalar deleting destructor for FishingProcessor. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00ccf890` | `ProxiedFishingPlayer::Ctor` | ProxiedFishingPlayer | Constructs a proxy representation of a fishing participant. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00ccf9c0` | `ProxiedFishingPlayer::Dtor` | ProxiedFishingPlayer | Destroys a proxied fishing player and its owned resources. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00ccfbd0` | `ProxiedFishingPlayer::ScalarDeletingDtor` | ProxiedFishingPlayer | MSVC scalar deleting destructor for ProxiedFishingPlayer. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00cd0150` | `ControllerFishing::UpdateFrame` | ControllerFishing | Main native per-frame update for local fishing presentation and controller state. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |

## 🟢 likely (21)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b62eb0` | `StorePos4AndDistanceFromPrev` |  | Position bookkeeping for one fishing object (bobber or school): it keeps the current 4-float position and caches the distance from the previous one, called only from the fishing packet dispatcher FUN_00b6cd80. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b640e0` | `BuildLaunchVector4` | FishingProcessor | Cast/launch vector maths for the fishing processor: it turns a target point into the velocity-like vector the bobber is sent along, with the angle computed only in state 5 (hook/reel per the fishing notes). | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b64300` | `FishingProcessor::FindOrCreateProxiedPlayer` | FishingProcessor | Lookup-or-create for the processor's proxied fisherman records, keyed by the 8-byte actor handle; seven callers (state changes, per-frame updates and packet handling) use it. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b646e0` | `FindSchoolByKey` | FishingProcessor | Keyed lookup in the processor's proxied fishing school list (the list whose 0x48-byte nodes the ProxiedFishingSchool helpers in this shard erase and clear). | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b64710` | `FindUnderwaterFishByKey` | FishingProcessor | Keyed lookup in the processor's proxied underwater fish list; the node layout (0x1a0 bytes, links at +0x190/+0x194) matches ProxiedFishingUnderwaterFish. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b65460` | `FishingProcessor::TouchLocalProxiedPlayer` | FishingProcessor | Refreshes or marks the processor's record for the local player; called from the per-frame path FUN_00b6c290. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b67b80` | `UpdateFishingLureAttachments ⚠downgrade` |  | Connects fishing state changes to named lure-model attachment points. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b68600` | `AppendResolvedActorValue` |  | Builds a resolved list used by the fishing processor path. | [[Native Map/Shards/S0350 00b66b80-00b6b400|S0350]] |
| `00b6bae0` | `CopyAssign_List50_Map60` |  | Copies the fishing registration response record decoded by the sub-3 case of FUN_00b6cd80 into the destination record. | [[Native Map/Shards/S0351 00b6b510-00b6c290|S0351]] |
| `00b6bb90` | `FishingProcessor::SetActive` | FishingProcessor | Enters or leaves fishing for the local client: audio save and restore, local registration and music on entry; state, school and sound teardown on exit. | [[Native Map/Shards/S0351 00b6b510-00b6c290|S0351]] |
| `00b6c290` | `FishingProcessor::Update` | FishingProcessor | Per-frame update of the fishing processor: proxied player updates and the fishing request state machine; called from PlayClient::Update each frame. | [[Native Map/Shards/S0351 00b6b510-00b6c290|S0351]] |
| `00ccecf0` | `ControllerFishing::UpdateFrame` | ControllerFishing | Advances the native fishing controller's world activity state each frame. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00ccf260` | `FishingController::ResetResultState → ControllerFishing::ResetResultState` | ControllerFishing | Resets fishing result presentation state after a completed activity. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00ccf3c0` | `ControllerFishing::SetState` | ControllerFishing | Applies native fishing controller state transitions and related UI/world effects. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00ccf6d0` | `FishingProcessor::SetCastFlags` | FishingProcessor | Records one of two cast-related flags from fishing processor updates. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00ccf6f0` | `FishingProcessor::SetPlayerState` | FishingProcessor | Updates the fishing processor's player state and four-component vector. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00ccf720` | `ProxiedFishingPlayer::ReleaseResources` | ProxiedFishingPlayer | Releases proxied fishing player presentation objects. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00ccfb00` | `ProxiedFishingPlayer::SetAnimation` | ProxiedFishingPlayer | Applies a fishing animation state to a proxied player. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00ccfbf0` | `ProxiedFishingPlayer::ShowResult` | ProxiedFishingPlayer | Builds result UI for an observed fishing outcome; it does not grant the catch or reward. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00ccffb0` | `FishingProcessor::SetState` | FishingProcessor | Applies proxied fishing state transitions and presentation changes. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00cd0df0` | `FishingSchool::RefreshModelEffects` |  | Applies a shared transform or effect value to resolved fishing school models. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |

## 🟡 speculative (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b616e0` | `CallSlot2On0x32C ? ⚠downgrade` |  | Called from FUN_00cd0150, the bbe_fishing_* bobber/school model factory, so it releases or advances the sub-object stored at +0x32c of the fishing object. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b62f30` | `StoreTwoVec4BlocksAndScalars ? ⚠rename` |  | Fills a two-block record used by the fishing path/school data flow: it is called from the packet dispatcher FUN_00b6cd80 and from FUN_00cd0150, the engine-side bbe_fishing model factory. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00ccf7b0` | `FishingProcessor::ValidatePlayerAction ? ⚠downgrade` | FishingProcessor | Validates whether a proxied fishing player can perform the requested action. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00ccf820` | `FishingProcessor::IsPlayerReady ? ⚠rename` | FishingProcessor | Checks whether the fishing player can proceed to the next activity phase. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00ccf860` | `ProxiedFishingPlayer::ReleaseHandle ?` | ProxiedFishingPlayer | Releases one actor-owned resource held by a proxied fishing player. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |

## ⚪ unknown (8)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b64360` | `— ⚠reject` | FishingProcessor | Enters and leaves the underwater view/audio state of the fishing activity, including the script notifications the UI reacts to. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b64580` | `— ⚠reject` | FishingProcessor | State machine entry for the fishing processor: it switches the cast/hook/reel states and drives the matching animation, sound and underwater transitions. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b65200` | `— ⚠reject` | FishingProcessor | Rebuilds the composite effects attached to fishing school models, most likely when a zone or asset finishes loading; no caller is recorded, so the trigger stays unknown. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b65390` | `— ⚠reject` |  | Applies a uniform scale to the resolved object (the bobber or fish proxy) and enables the corresponding render flag; called from the per-frame bobber update FUN_00b65fb0. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b65ed0` | `— ⚠reject` | FishingProcessor | Teardown of the processor's fish entry records (the client fish table), including the per-entry alias strings and the two boolean flags that FUN_00b63740 copies into list members. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b65fb0` | `— ⚠reject` | FishingProcessor | Per-frame presentation update of the fishing bobber and its line while casting, hooking and reeling, driven by the state machine field +0x1c. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00ccf770` | `— ⚠reject` | FishingProcessor | Enables presentation for a resolved fishing actor. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |
| `00ccfdb0` | `— ⚠reject` | ProxiedFishingPlayer | Creates and starts fishing result presentation for a proxied player. | [[Native Map/Shards/S0417 00ccbd60-00cd11e0|S0417]] |

