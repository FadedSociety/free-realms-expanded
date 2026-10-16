# System: anim.controller

20 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00788ad0` | `AnimTrackLogText` |  | Builds the per-track diagnostic text (track, anim, animGroup, filterType, priority, speed, elapsed time, ease state) that the play-logging path of 00789010 writes before and after a play. | [[Native Map/Shards/S0169 00787e70-0078b5f0|S0169]] |
| `00789010` | `Actor::PlayAnimRecord` | Actor | Starts one animation request (record at param_2) on the Actor. It resolves the transition rule and slot, plays the new animation or defers it until loaded, and logs the reason when it cannot play. | [[Native Map/Shards/S0169 00787e70-0078b5f0|S0169]] |
| `0078a980` | `ListAnimationReferenceDefinition::DerivedDtorBody` |  | Destructor body of an animation-definition holder whose primary base is the List of AnimationReferenceDefinition; it runs from unwind cleanup code. | [[Native Map/Shards/S0169 00787e70-0078b5f0|S0169]] |
| `00a6ee70` | `ControllerSingleRotation::~ControllerSingleRotation` | ControllerSingleRotation | Non-deleting destructor body of ControllerSingleRotation. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6ef10` | `ControllerSingleRotation::ScalarDeletingDtor` | ControllerSingleRotation | Deleting-destructor implementation of ControllerSingleRotation. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |

## 🟢 likely (9)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0078a460` | `ApplyAnimRecord` |  | Applies an animation record to this object and its eligible children. The anim meaning rests on its callee 00789010, whose strings name the animation log messages. | [[Native Map/Shards/S0169 00787e70-0078b5f0|S0169]] |
| `0078b2e0` | `Actor::PlayAnimRecordAndChildren` | Actor | Plays an animation record on this Actor through 00789010 and repeats it for child actors. The name rests on the callee 00789010 (its anim strings) and on the Update-to-0078b520 ECX chain. | [[Native Map/Shards/S0169 00787e70-0078b5f0|S0169]] |
| `0078b3b0` | `ForEachChildApplyAnimRecord` |  | Applies a copy of a resolved animation record to each eligible child. The anim meaning rests on its callees 0078a460 and 007829a0. | [[Native Map/Shards/S0169 00787e70-0078b5f0|S0169]] |
| `0078b520` | `Actor::LookupAndPlayRecord` | Actor | Looks up an entry by id and plays its record through 0078b2e0. Actor::Update calls it with (-1, 1). | [[Native Map/Shards/S0169 00787e70-0078b5f0|S0169]] |
| `0096c780` | `PlayOrQueueActorAnimation` |  | Plays an actor animation immediately, logging begin and end lines to AnimMixer.log, or queues the id and duration at this+0x3f0/+0x3f4 for later playback (0096d370 consumes them with flag 0). | [[Native Map/Shards/S0258 00969a10-0096de80|S0258]] |
| `0096cd80` | `DumpAnimationsToMixerLog` |  | Diagnostic dump of an actor's animation set to AnimMixer.log, bracketed by begin and end lines. | [[Native Map/Shards/S0258 00969a10-0096de80|S0258]] |
| `0096cfe0` | `ListAnimationsDiagnostic` |  | Diagnostic listing of an actor's animations, bracketed by begin and end lines. | [[Native Map/Shards/S0258 00969a10-0096de80|S0258]] |
| `00a6eda0` | `ControllerSingleRotation::ControllerSingleRotation ⚠downgrade` | ControllerSingleRotation | Constructs a single-axis orbit controller whose angle advances by 2*pi per period (+0xec, default 15000). | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00a6eec0` | `ControllerSingleRotation::ConsumeFullTurn` | ControllerSingleRotation | Full-turn check for the orbit angle: reports one completed revolution and wraps the angle. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |

## 🟡 speculative (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0078a740` | `NodeValueOverIdDuration ?` |  | Returns a float. It returns 0 when 007829a0(id) is below 1 or no node matches. Otherwise it finds the node keyed by that id in the hash table at this+0x3e8 (mask this+0x3d8), sets the node object's +0x60 float from 0077… | [[Native Map/Shards/S0169 00787e70-0078b5f0|S0169]] |
| `0078b390` | `Wrapper_0078a460 ?` |  | Forwards its two stack arguments to 0078a460 as (arg1, 0, arg2), leaving ECX unchanged so the caller's receiver passes through. Returns with ret 8. | [[Native Map/Shards/S0169 00787e70-0078b5f0|S0169]] |
| `0078b5a0` | `SehCallDtorBody_0078a980 ?` |  | Installs an SEH frame and calls 0078a980 with this in ECX and no stack argument. The DAT_01b839d8 ^ (uint)local_10 value in the pseudocode is the /GS stack cookie, not a call argument. Removes the frame and returns. | [[Native Map/Shards/S0169 00787e70-0078b5f0|S0169]] |
| `0096d1b0` | `PlayAnimationRequestWithMixerClear ?` |  | Thiscall with nine stack parameters. When param_2 is nonzero it builds an IString list, uses it only when the byte at this+0x145 bit 7 is set and the owner id comparisons pass, passes a request struct built from params … | [[Native Map/Shards/S0258 00969a10-0096de80|S0258]] |
| `00a6eef0` | `ControllerSingleRotation::ForwardFloatToSharedSlot70 ?` | ControllerSingleRotation | Virtual slot 28 (+0x70), 29 bytes: gets an object from 0094c6f0 and, when it is non-null, calls that object's virtual slot 28 (+0x70) with the float argument. Returns nothing. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |

## ⚪ unknown (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a6ef70` | `— ⚠reject` | ControllerSingleRotation | Per-frame update of the single-axis orbit controller: moves a point around the centre supplied by the manager object. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |

