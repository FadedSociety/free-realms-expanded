---
type: build-guide
section: Gameplay
updated: 2026-10-12
---

# Soccer

Part of [[Build Guides]] › Gameplay. Related: [[Minigame Framework]], [[Physics]].

<!-- NARRATIVE START -->
## How it works

### How to read this

The address citations point to accepted labels in `lab/native-map/run/map.sqlite`. `confirmed` means the cited native client body supports the description; pseudocode still needs byte checks where the label is `unknown` or `speculative`. **OSFR-held data (packet captures and server JSON) is not used for facts.** Older soccer traces and derived server observations are therefore excluded from protocol, tuning, roster and reward claims here. No native Soccer match run was inspected for this task.

`R` means reconstructed server policy. A typed client input does not prove that a proposed server rule or output is historically correct. The Rust service uses bounded, configurable policy and labels its bot fill, physics, clock, results and reward grant as `R`.

### Overview

Soccer is a native client activity with its own `SoccerProcessor`, `SoccerPlayer` and `SoccerBall` code. The minigame framework names Soccer as type `12`; it does not imply that Soccer is a Flash movie. The client has a Soccer-specific UI state handler for waiting, ready, halftime, goals, sudden death, the timer and results. Its state-to-packet mapping is not fully recovered.

The Rust service now joins that native activity to the shared `MinigameRuns` result lifecycle. That is an integration choice (`R`): the available native client evidence establishes input readers and result UI, but not Soccer's original match launch, terminal packets or reward transaction. The service owns team assignment, ball state, clock, goals and full-time result; no client packet can declare a score.

### How the client implements it

- `SoccerProcessor::DispatchFamily76Packet` (`00b80ac0`, confirmed) routes the Soccer family-76 packet classes. `SoccerPlayer::SendUpdatePlayerState` (`00cdc5c0`, confirmed) writes the 62-byte sub-14 state body. `SoccerPlayerBase::SetState` (`01072020`, confirmed) updates the state and increments the client state-change counter.
- `ControllerSoccer::ApplyInputActions` (`00afb4a0`, confirmed) consumes controller action entries. Its bit field is not the kick/pass vector. The client sends the action direction through sub 14, while sub 16 controls carry two floats and a raw `u32` input word.
- `SoccerBall::ApplyOrSendImpulse` (`00cde9b0`, confirmed) and the sub-13 reader show the impulse layout, but do not establish an authoritative server emission policy. The located action-direction path (`01074b50`) has an `unknown` map label; its bytes were checked with `nmq dis`, and the separate direction-semantics review confirms the relevant dataflow. The action direction is normalized by `SoccerPlayerBase::NormalizeIntoSlot50` (`01072610`, confirmed).
- `SoccerProcessor::UpdateTimerView` (`00b7be70`) feeds the Soccer timer UI. `SoccerProcessor::UpdateScoreboard` (`00b7c070`) updates the two displayed team scores. `ApplySoccerUiState` (`00b7de60`) invokes `SoccerHalftimeBoard`, `SoccerGoal`, `SoccerSuddenDeath`, `SoccerTimer` and `SoccerResults` callbacks. These UI strings do not recover the server's state packet values or match transition rules.
- The shared minigame manager still owns the type-12 record and common score-screen machinery. The native Soccer path has its own UI, so the Rust service's generic 39/18, 39/47 and 39/19 result sequence remains a testable compatibility candidate (`R`), not a recovered Soccer sequence. See [[Minigame Framework]].

### Data and formats

The client type registry assigns Soccer type `12`. Soccer's class family is `76` (`0x4c`); the gateway tunnel wraps an inner family/sub-id and bounded payload. The mapped sub-13, sub-14 and sub-16 readers consume little-endian fields and reject short reads. This guide names semantic fields only where client code establishes them; remaining float4s and packet words retain wire names.

The client has Soccer player/team UI state and score display paths, including the native `UpdateScoreboard` callback. Those readers do not supply score formulas, stat columns or award amounts. No asset conversion or replacement is part of this service task.

### Network

Offsets below are relative to each Soccer packet payload, after the gateway tunnel and family/sub-id. They come from the native client readers and sender dataflow, not from the excluded packet captures.

| Sub-id | Client layout | Client evidence | Rust handling |
|---|---|---|---|
| 14 `UpdatePlayerState` | `u64` player at `0x00`; `u8` state at `0x08`; quantized charge `u8` at `0x09`; state-change counter `u32` at `0x0a`; vecA/action direction `f32[4]` at `0x0e`; vecB steering/movement `f32[4]` at `0x1e`; vecC position-like value `f32[4]` at `0x2e`. Total: 62 bytes. | Reader `00b794b0`; sender `00cdc5c0`; setter/counter `01072020`; direction consumer `01072610`. | Authenticate sender, use the transport sequence rather than the state counter for replay protection, validate finite direction and charge, require possession, then let server physics decide the ball result. Passes require a server-verified teammate target. |
| 16 `UpdatePlayerControls` | `u64` player at `0x00`; `f32[2]` direction at `0x08`; raw `u32` controls at `0x10`. Total: 20 bytes. | Reader `00b793c0`; action consumer `00afb4a0`. | Treat direction and bits as advisory input. No bit is identified as kick or pass. |
| 13 `ImpulseBall` | `u32` at `0x00`; `f32[4]` at `0x04`; `f32` at `0x14`; `f32` at `0x18`; `u32` at `0x1c`; `f32` at `0x20`. Total: 36 bytes. | Reader `00b79580`; client method `00cde9b0`. | The Rust candidate emission maps direction/charge/power/action using `R` semantics. This is not a recovered server rule or captured output. |

Sub 14's `u32` is a client state-change counter: the client setter increments `player+0xb0` when state changes. It is not the service's transport sequence. The aux byte encodes the client charge ratio using `255.0` and decodes with `1/255`. vecA is the action direction; vecB and vecC remain untrusted input. A client `GoalMade` packet is refused by the Rust authority.

The native map establishes the state-byte field but not the full kick/pass state table. `soccer_match::classify_state` currently uses an explicit candidate allowlist (R); its state ranges must not be treated as a recovered retail enum.

The Rust authority's coordinate system, pitch bounds, kick-power scale, movement equations, goal crossing, out-of-play reset, kickoff, side switch and half length are all `R` inputs. The 180-second default is a convenience value, not a client-recovered duration. A client timer reader proves that the UI consumes a supplied time; it does not prove a fixed timer length.

### What the Rust server must do

- Register authenticated sessions and assign teams/slots on the server. Four slots per team are an `R` roster limit for this local slice. `fill_bots_for_solo` fills the remaining slots with passive synthetic bot identities (`R`); these bots have no AI and do not score.
- Offer type 12 through `MinigameRuns`, handle its start packet, and begin Soccer only after connected humans reach the Playing state. The encounter mode and the shared result route are an `R` adapter because Soccer's original lifecycle packets are not established here.
- Keep the ball, possession, score, clock, goals and full-time transition server-owned. Validate the authenticated sender, finite values, movement/time budgets, state classification, charge, possession and pass target. Never accept a client score, ball transform or impulse as authority.
- On halftime, require the explicit second-half transition. On full time, derive the result from the server score and send the generic result sequence through `MinigameRuns`. A tie currently counts as a loss for the per-player generic result (`R`).
- Pass only an explicitly configured `RewardGrant` to `rewards::apply_grant` for a winner. The stable run key prevents a second payment. No Soccer reward amount is recovered; the grant is `R` data supplied by the caller.
- On disconnect or gateway timeout, cancel that player's active generic run without reward, remove their roster entry and release held possession. A solo match with no human sessions is cancelled. The gateway, not this service, chooses the timeout threshold (`R`).

These rules are unit-tested but do not establish native playability, visual ball synchronization or an original-client Soccer results screen.

### Video and wiki evidence

The read-only content query `python -I ../lab/content/tools/cq.py vid-grep 'Soccer Gate'` returns overworld frames with a Soccer Gate waypoint. It contains no Soccer match, score or result observation. `python -I ../lab/content/tools/cq.py wiki 'Soccer Star'` reports that the job can play minigames solo or in teams and describes Soccer Star progression. That is fan-wiki context only; it does not establish the native packet contract, roster size or rewards.

### Open questions

- Which type-12 info record, launch exchange and zone-entry values start a native Soccer match? Which generic lifecycle packets does the original use?
- How does the client `UpdateGameState` word array map to wait, ready, halftime, goal, sudden-death and results UI states?
- What are the original field dimensions, movement and collision rules, kick/power scale, goal test, half duration, side-switch rule and overtime policy?
- What do the sub-13 fields mean at the Soccer subsystem boundary, and which path emits a server impulse?
- How many players fit each team? What are the original disconnect, timeout, draw, results-close, Soccer Star XP and reward rules?
- Do family-76 result fields and the generic `MinigameRuns` results coexist in a retail Soccer session? Only an original-client end-to-end run can settle that integration.

### Key functions

| Address | Native map label | Relevance |
|---|---|---|
| `00b80ac0` | `SoccerProcessor::DispatchFamily76Packet` (confirmed) | Routes Soccer family-76 messages. |
| `00b794b0` | `SoccerPacketUpdatePlayerState::ReadBody` (confirmed) | Reads sub-14 fields and float4 widths. |
| `00cdc5c0` | `SoccerPlayer::SendUpdatePlayerState` (confirmed) | Writes the client sub-14 input. |
| `01072020` | `SoccerPlayerBase::SetState` (confirmed) | Changes state and increments its counter. |
| `01074b50` | unknown label; bytes inspected with `nmq dis` | Client action/charge path; only the independently reviewed field dataflow is relied on. |
| `01072610` | `SoccerPlayerBase::NormalizeIntoSlot50` (confirmed) | Normalizes vecA's direction into the player action slot. |
| `00b793c0` | `SoccerPacketUpdatePlayerControls::ReadBody` (confirmed) | Reads sub-16 direction and raw control word. |
| `00b79580` | `SoccerPacketImpulseBall::ReadBody` (confirmed) | Reads the 36-byte sub-13 layout. |
| `00b7be70` / `00b7c070` | `SoccerProcessor::UpdateTimerView` / `UpdateScoreboard` (confirmed) | Connects supplied time and scores to UI. |
| `00b7de60` | `ApplySoccerUiState` (confirmed) | Dispatches Soccer halftime, goal, timer and results UI callbacks. |
<!-- NARRATIVE END -->

## Mapped functions (generated)

206 accepted functions: 93 confirmed, 63 likely, 24 speculative, 26 unknown. Network-linked (`wire`): 0.

Subsystems: `minigame.soccer` (190), `ui.soccer` (16)

### `minigame.soccer`

**ControllerSoccer** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00afb070` | ControllerSoccer::Dtor | confirmed | Fastcall destructor body (ECX = this): stores ControllerSoccer::vftable at +0, sets the SEH state to -1, calls base Controller destructor 0094fab0 and returns.… |
| `00afb230` | ControllerSoccer::ScalarDeletingDtor | confirmed | Thiscall scalar deleting destructor body: stores ControllerSoccer::vftable at +0, calls base destructor 0094fab0, then frees this with _free when bit 0 of the … |
| `00afb4a0` | ControllerSoccer::ApplyInputActions | confirmed | Thiscall virtual at ControllerSoccer vtable slot 50. Walks the entry list at this+0xc8 (owner = this+0x74): on kind-0 entries (flag +0xd) id 0x13 calls 009cf43… |
| `00afb7a0` | ControllerSoccer::ControllerSoccer | confirmed | Thiscall ctor, three stack args forwarded to base 0094f6c0, returns this (ECX=this). Stores ControllerSoccer::vftable at +0x00, initialises about 0x7e dwords o… |
| `00afb0d0` | ControllerSoccer::SetF0 | speculative | Virtual at ControllerSoccer vtable slot 23: stores the dword argument at this+0xf0. |
| `00afb0f0` | ControllerSoccer::GetF0 | speculative | Getter at ControllerSoccer vtable slot 24: returns the float at this+0xf0 as float10 (bytes d9 81 f0 00 00 00 c3). |
| `00afb100` | ControllerSoccer::SetF4 | speculative | Virtual at ControllerSoccer vtable slot 26: stores the dword argument at this+0xf4. |
| `00afb120` | ControllerSoccer::GetF4 | speculative | Getter at ControllerSoccer vtable slot 27: returns the float at this+0xf4 as float10 (bytes d9 81 f4 00 00 00 c3). |
| `00afb130` | ControllerSoccer::VcallA4AfterBaseCall | speculative | Fastcall virtual at ControllerSoccer vtable slot 9. Calls 0094e210 with no arguments, then virtual call +0xa4 on this with the global constant at 0x01814c18. S… |
| `00afb400` | ControllerSoccer::SetVec4FromAxisAngle | speculative | Virtual at ControllerSoccer vtable slot 28: calls FUN_00770e70 with a local constant vector (0, 1.0, 0, 0) and the float argument, scales the returned matrix t… |
| `00afbc60` | — | unknown | Thiscall with one int arg (time step), vtable slot 51 (+0xcc). Returns unless the object at this+0x74 -> +0x120 -> +0x21c is nonzero. Reads 01071fc0 and sets f… |

**SoccerBall** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cdcda0` | SoccerBall::CleanupAttachedHandles | confirmed | Calls 0107ba90, adjusts ECX to this, then tail-jumps to 00cdcbe0 for the five-handle cleanup. |
| `00cdcdb0` | SoccerBall::SetAttachedStateBits | confirmed | Gets a component through this+0xc0, updates bits 0 and 1 in its word at +0x10 from param_2, invokes the matching virtual callbacks, and forwards param_2 to 010… |
| `00cdcfd0` | SoccerBall::EnsureHandle164 | confirmed | When handle +0x164 is empty and the descriptor at +0x160 is ready, obtains its resource id, creates and stores a handle, then invokes handle vtable slot 7. |
| `00cdd9e0` | SoccerBall::DestructorBody | confirmed | Restores SoccerBall subobject vtables, releases attached handles and six embedded strings, destroys the asset requester and base object, and runs shared cleanu… |
| `00cddde0` | SoccerBall::FillFourDwordOutput | confirmed | Gets the nested object through this+0xc0; fills four output dwords from its fields, or delegates to 007652e0 when its +0x1a0 field is set, with a fallback when… |
| `00cdde90` | SoccerBall::FillThreeDwordOutput | confirmed | Gets the nested object through this+0xc0; fills three output dwords from its fields, or delegates to 007652e0 when its +0x1a0 field is set, with a fallback whe… |
| `00cddf40` | SoccerBall::ScalarDeletingDestructor | confirmed | Calls the SoccerBall destructor body, frees this only when flag bit 0 is set, and otherwise returns this. |
| `00cde1c0` | SoccerBall::Constructor | confirmed | Initializes the proxied base, installs SoccerBall and embedded subobject vtables, stores constructor pointers, zeros handle/state fields, initializes embedded … |
| `00cdd100` | SoccerBall::UpdateFiveEffectSlots | speculative | Calls 0107ba30, then for positive counts at five slots submits each slot id, its type from 2 through 6, and this+0x114 to the effect manager. |
| `00cdd200` | — | unknown | Copies sixteen dwords from param_2; when param_3 is zero, blends three matrix columns with 0107bb00 using 1.0 minus this+0x128, then applies the matrix through… |
| `00cdd300` | — | unknown | Updates ball effects from state and threshold checks, normalizes and smooths motion fields at +0x68 through +0x6b, then synchronizes dependent transforms throu… |

**SoccerBallBase** (17)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cddd80` | SoccerBallBase::GetFieldB0 | confirmed | Returns the dword stored at this+0xb0. |
| `0107ba30` | SoccerBallBase::ResetStateDispatch | confirmed | Thiscall with one stack argument (ret 4): sets the dword at +0x14 to -1, calls virtual slot [9] (+0x24) on this with the argument, sets the float at +0x74 to 0… |
| `0107ba70` | SoccerBallBase::AcquireOnce | confirmed | Fastcall (this in ECX). If the byte at +0x70 is zero: when the handle at +0x48 is non-null, calls 00fda940(handle) with ECX set to the object at +0x44; then se… |
| `0107ba90` | SoccerBallBase::ReleaseHandleIfLocked | confirmed | Fastcall (this in ECX). If the byte at +0x70 is nonzero: when the handle at +0x48 is non-null calls 00fd80f0(handle) with ECX set to the object at +0x44, then … |
| `0107bae0` | SoccerBallBase::ForwardArgToHandleIfSet | confirmed | Thiscall with one stack argument (ret 8, two dwords popped). When the handle at +0x48 is non-null calls 00fe2850 with ECX = the handle and the argument; otherw… |
| `0107bb60` | SoccerBallBase::SetFlag18 | confirmed | Thiscall with one byte stack argument (ret 4): stores the byte into this+0x18. |
| `0107bb70` | SoccerBallBase::TailSlot12IfUnlocked | confirmed | Fastcall (this in ECX). If the byte at +0x70 is zero, tail-jumps to virtual slot [12] (+0x30) of this; otherwise returns 0. |
| `0107bb80` | SoccerBallBase::SoccerBallBase | confirmed | Thiscall constructor with one stack argument (stored at +0x10): sets SoccerBallBase::vftable; sets the dword at +0x14 to -1; zeroes the bytes at +0x18, +0x19 a… |
| `0107bc80` | SoccerBallBase::Destruct | confirmed | Fastcall destructor body: stores SoccerBallBase::vftable; if the flag at +0x70 is set calls 00fd80f0 on the handle at +0x48 and clears the flag; clears the byt… |
| `0107bce0` | SoccerBallBase::ClampStoreAt74DivideTo78 | confirmed | Thiscall with a dword argument and two float arguments. When the handle at +0x48 is non-null: if the flag at +0x70 is zero, zeroes a 16-byte local and calls 00… |
| `0107bd90` | SoccerBallBase::FillQuadFromHandle | confirmed | Thiscall with one pointer argument to four dwords (ret 4, returns the argument). When the handle at +0x48 is non-null calls 00fe2700 on it and copies the dword… |
| `0107bdf0` | SoccerBallBase::FillTripleFromHandle | confirmed | Thiscall with one pointer argument to three dwords. Writes zero to all three; when the handle at +0x48 is non-null, calls 00fe2700 on it (ECX = handle) and cop… |
| `0107bee0` | SoccerBallBase::FillPairFromB0OrDefault | confirmed | Thiscall with one pointer argument to two dwords. When the dword at +0xb0 is non-null writes its dwords at +0x10 and +0x14 of that object into the output; othe… |
| `0107bf30` | SoccerBallBase::PairViaSlot13IfUnlocked | confirmed | Thiscall with one pointer argument to two dwords. If the byte at +0x70 is zero, calls virtual slot [13] (+0x34) with the output; otherwise writes the constants… |
| `0107bf70` | SoccerBallBase::ScalarDeletingDtor | confirmed | Thiscall with one stack byte argument (ret 4): calls 0107bc80 (SoccerBallBase::Destruct) on this, then calls _free(this) through thunk 00d82463 when bit 0 of t… |
| `0107c2e0` | SoccerBallBase::ReinitHandlesFromArgs | confirmed | Thiscall with eight stack arguments (ret 0x20). Releases the previous state: calls virtual slot [4] (+0x10) when the flag at +0x70 is set, releases the handle … |
| `0107bf90` | — | unknown | Thiscall with one float argument (step time). When the byte at +0x18 is set: advances the timer at +0x40 by the step; decreases the value at +0x3c when it exce… |

**SoccerPickup** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cd6a70` | SoccerPickup::SoccerPickup | speculative | Stores the fourth argument at +4, installs the SoccerPickup vtable, clears +8 and +1c, stores the third argument at +0xc, and stores the second argument and it… |
| `00cd6aa0` | SoccerPickup::DeletingDestructor | speculative | Calls the SoccerPickup destructor body, tests flag bit zero, conditionally frees the object, and returns the original object pointer. |
| `00cd6a30` | — | unknown | Restores the SoccerPickup vtable, releases its optional resource reference, and invokes slot 2 on the stored associated object when present. |

**SoccerPlayer** (35)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cd7fd0` | SoccerPlayer::IsFloatFieldNonpositive | confirmed | Returns one when the float at this+0xac is less than or equal to zero, otherwise returns zero. |
| `00cd7ff0` | SoccerPlayer::GetBallWhenAvailable | confirmed | When the controller and its nested ball state are nonzero, calls the shared ball getter and returns its result; otherwise returns zero. |
| `00cd8010` | SoccerPlayer::ForwardFloatThroughMappedMethod | confirmed | Loads the supplied float, places it on the floating-point stack, forwards it to 01073f20, and returns that helper's result. |
| `00cd8020` | SoccerPlayer::ResetVectorFromField100 | confirmed | Copies this+0x100 into this+0x30, clears this+0x34 and this+0x3c, then sets this+0x38 to the global constant minus this+0x104. |
| `00cd8050` | SoccerPlayer::CallTrackedEntityHelperAndReturnArgument | confirmed | Calls 00b77480 with the supplied argument, then returns that same argument unchanged. |
| `00cd8080` | SoccerPlayer::CopyFourFloatDirection | confirmed | Copies four 32-bit values from this+0xf0 through this+0xfc into the output buffer. |
| `00cd80b0` | SoccerPlayer::GetGlobalLookupField | confirmed | Calls the shared lookup helper, reads the resulting object's field at +0x90, and returns that value. |
| `00cd8330` | SoccerPlayer::HasNearbyActiveRecord | confirmed | Checks up to three records and returns one when an active record's horizontal squared distance is within the configured threshold; otherwise returns zero. |
| `00cd89b0` | SoccerPlayer::UpdateMappedResource | confirmed | Calls 01071f90, processes the six resource entries, and conditionally invokes slot 0x0c on the object at this+0x400. |
| `00cd89e0` | SoccerPlayer::UpdateMappedRecordState | confirmed | Guards on controller and record state, passes the supplied argument through virtual calls, may call 00b75570, copies transform data, and invokes companion slot… |
| `00cd8b00` | SoccerPlayer::WriteMappedTransform | confirmed | Copies four transform values into the mapped record and updates the associated companion object. |
| `00cd8be0` | SoccerPlayer::GetControllerField1E8 | confirmed | Returns the nested controller object's field at +0x1e8 when the object exists, otherwise returns one. |
| `00cd8c00` | SoccerPlayer::GetControllerField148 | confirmed | Returns the nested controller object's field at +0x148 when the object exists, otherwise returns one. |
| `00cd8c20` | SoccerPlayer::OnStateChanged | confirmed | When the requested state differs from this+0x60, updates the state field, maps action/state codes, rebuilds animation state, and updates duration at +0xac. |
| `00cd8f50` | SoccerPlayer::SetHorizontalDirection | confirmed | Calls 01074590, checks the squared length of a vector, performs a lookup when the threshold passes, and sends the accepted direction to 0095dae0. |
| `00cd9010` | SoccerPlayer::GetControllerVector | confirmed | When the controller object is present, copies four values from its fields at +0x5b4, +0x5bc, +0x5c0, and +0x5c8; otherwise returns the fallback values. |
| `00cd9130` | SoccerPlayer::GetMappedValue6E4 | confirmed | Returns the mapped object's field at +0x6e4 when lookup succeeds, otherwise returns one. |
| `00cd9180` | SoccerPlayer::GetMappedValue6E8 | confirmed | Returns the mapped object's field at +0x6e8 when lookup succeeds, otherwise returns one. |
| `00cd91d0` | SoccerPlayer::GetNestedControllerObject | confirmed | Returns the nested controller pointer at +0x30 when available, otherwise returns zero. |
| `00cd9200` | SoccerPlayer::BuildOrientationMatrix | confirmed | Reads angles at +0x610 and +0x618, builds a rotation matrix through 00770e70, and appends the output vector to the resulting matrix. |
| `00cd9570` | SoccerPlayer::SetActionResource | confirmed | Returns early when this+0x600 is set; otherwise stores the supplied values and dispatches action codes 1 through 5 to resource-entry virtual calls. |
| `00cd9af0` | SoccerPlayer::OnStateChanged | confirmed | Handles SoccerPlayer state changes by updating resource entries and performing state-specific pickup/action transitions. |
| `00cd9df0` | SoccerPlayer::GetTargetDirectedVector | confirmed | Scans the roster for another qualifying entity, converts its world position to field space, computes and normalizes a horizontal direction from this player, an… |
| `00cda080` | SoccerPlayer::InitializeAnimationAssets | confirmed | Looks up twelve MG_Soccer animation/effect asset names and initializes a corresponding referenced object handle for each entry. |
| `00cdb1a0` | SoccerPlayer::~SoccerPlayer | confirmed | Restores SoccerPlayer vtables, unregisters controller references, releases resource slots, updates and clears the overlay element, releases paired references a… |
| `00cdbea0` | SoccerPlayer::ScalarDeletingDestructor | confirmed | Calls the SoccerPlayer destructor body, frees this only when flag bit 0 is set, and otherwise returns this. |
| `00cd7a70` | SoccerPlayer::UpdateTimedResources | likely | Updates a timed state flag and processes the resource entries held at offsets 0x3d0 and 0x3d8. |
| `00cd7c30` | SoccerPlayer::ReleaseResourceReferencePairs | likely | For each of twelve resource pairs, invokes the paired object with argument one when present, decrements a positive reference count, and clears the primary hand… |
| `00cd8640` | ReleaseSoccerPlayerResources | likely | Processes the six-entry group, then calls the release helper for eleven resource fields held by the SoccerPlayer object. |
| `00cd8700` | SoccerPlayer::UpdateResourceSlots | likely | Looks up resource slot data, advances its time value through 0078fcc0, and conditionally updates the associated matrix through the nested +0x10 interface. |
| `00cd8e60` | SoccerPlayer::RunConditionalUpdateHelpers | likely | Calls 00b75ac0 when its state guards pass and then invokes 01071a00. |
| `00cdb370` | SoccerPlayer::UpdateDebugDisplay | speculative | Updates a timer and motion-related fields, then conditionally formats animation, server, game, player, ball, charge, and pickup values for display. |
| `00cd8f30` | — | unknown | Resolves the mapped record through 00cd8ec0, then passes the result to 01071af0. |
| `00cd9a00` | — | unknown | Builds a local event context from the controller at +0x14 and calls 010719b0 to emit it. |
| `00cda220` | — | unknown | Initializes the AssetRequester base and SoccerPlayer vtables, stores constructor inputs and the FreeFlyCamera identifier, zeros many fields, assigns local stat… |

**SoccerPlayerBase** (22)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01071990` | SoccerPlayerBase::ForwardSlot1Zero | confirmed | Virtual slot [25] of SoccerPlayerBase (+0x64). Tail-jumps to the object's virtual slot +4 with a zero stack argument. |
| `010719b0` | SoccerPlayerBase::ClearC4Link | confirmed | Virtual slot [27] of SoccerPlayerBase (+0x6c). When the pointer at this+0xc4 is nonzero, sets the dwords at this+0x124, this+0xc4 and this+0xa4 to zero; otherw… |
| `010719e0` | SoccerPlayerBase::ClearC4LinkAndCallChild | confirmed | Virtual slot [34] of SoccerPlayerBase (+0x88). When the pointer at this+0xc4 is nonzero it calls this's virtual +0x6c (slot [27], 010719b0 ClearC4Link) and the… |
| `01071af0` | SoccerPlayerBase::CopyFourDwordsTo20 | confirmed | Virtual slot [29] of SoccerPlayerBase (+0x74). Copies the four dwords at param_2 into this+0x20, +0x24, +0x28 and +0x2c. |
| `01071b10` | SoccerPlayerBase::PushControlInputs | confirmed | Virtual slot [40] of SoccerPlayerBase (+0xa0). Shifts the history of two inputs (+0x100/+0x104 to +0x10c/+0x110, the +0x108 value to +0x114), stores param_2, p… |
| `01071f90` | SoccerPlayerBase::ClearResourceDwords | confirmed | Virtual slot [22] of SoccerPlayerBase (+0x58). Zeroes the dwords at this+0x264, +0x270, +0x27c and +0x288. |
| `01071fe0` | SoccerPlayerBase::DtorBody | confirmed | Stores the SoccerPlayerBase vtable at [ECX] and returns. Does not free the object; it is the destructor body called from the EH unwind funclets and from the So… |
| `01072020` | SoccerPlayerBase::SetState | confirmed | Virtual slot [38] of SoccerPlayerBase (+0x98; SoccerPlayer inherits it). When the pointer at this+0xc4 is nonzero maps the requested state 1, 2, 8 or 9 to 0x3d… |
| `01072130` | SoccerPlayerBase::ScalarDeletingDtor | confirmed | Virtual slot [0] of SoccerPlayerBase. Stores the SoccerPlayerBase vtable, then calls the free thunk 00d82463 on this when bit 0 of its byte argument is set; re… |
| `01072610` | SoccerPlayerBase::NormalizeIntoSlot50 | confirmed | Virtual slot [31]. When the squared xyz length of the 4-float argument is above 0.1 (DAT_017e795c), normalises xyz with rsqrtps plus one Newton step, replaces … |
| `01074590` | SoccerPlayerBase::SetDirectionIfLongEnough | confirmed | Virtual slot [30], thiscall with one float4 pointer. When the squared length of the vector is at least DAT_017e795c (0.1), calls this vtable +0x7c (01072610) w… |
| `01075360` | SoccerPlayerBase::SoccerPlayerBase | confirmed | Thiscall constructor. Installs SoccerPlayerBase::vftable, stores its second argument at +0x10 and third at +0x14, zeroes the motion and state fields through +0… |
| `01071bc0` | SoccerPlayerBase::StoreDwordAt258 | likely | Virtual slot [24] of SoccerPlayerBase (+0x60). Stores param_2 in the dword at this+0x258. |
| `010727d0` | SoccerPlayerBase::OnStateStep | likely | Virtual slot [28], thiscall with (state, phase). Stores phase at this+0xa0. A switch on state-1 (jump table at 01073e18, entries for states 1..63; states 8, 9,… |
| `01075880` | SoccerPlayerBase::Tick | likely | Virtual slot [33]. Scales the step argument by DAT_017ebf0c, adds it into several float timers (for example this+0x138 and this+0x13c, the step itself) and dec… |
| `010726e0` | SoccerPlayerBase::NormalizeIntoSlot30 | speculative | Virtual slot [32]. When the squared xyz length of the 4-float argument is above 0.1, normalises xyz (rsqrtps, one Newton step, non-finite to 0). Either way it … |
| `01071a00` | — | unknown | Virtual slot [35] of SoccerPlayerBase (+0x8c). Stores param_2 at this+0xc4, calls its virtual +0x10, clears the byte at its +0x19, calls 0107be80(0, DAT_01762a… |
| `01071bf0` | — | unknown | Virtual slot [39] of SoccerPlayerBase (+0x9c; SoccerPlayer inherits it). Switches on an event code 0x17d5..0x1847 and returns a float from the DAT_01882xxx con… |
| `01072150` | — | unknown | Virtual slot [37]. Returns unless the handle at this+0xc4 is non-null and the byte at this+0xc0 is set. Takes a position through vtable +0x28, forms the offset… |
| `01073f20` | — | unknown | Virtual slot [36], thiscall with a float step argument. Builds the new position from this+0x20..0x2c plus the velocity block at this+0x30..0x3c scaled by this+… |
| `010748a0` | — | unknown | Fastcall on the player. Picks the source vector at this+0x30..0x3c (or this+0x50..0x5c when its squared length is at most DAT_017ebbb8). When this+0xe0 bit 0 i… |
| `01074b50` | — | unknown | Fastcall on the player, once per tick. Reads flag words at this+0x108 and this+0x114, accumulates the charge counter at this+0x224 and writes the charge ratio … |

**SoccerProcessor** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b759e0` | SoccerProcessor::SetTeamObject | confirmed | Virtual slot 1 of SoccerProcessor: picks the team slot (+0x618 for team 0, +0x630 for team 1, returning 1 immediately for other indices), releases the previous… |
| `00b7a370` | SoccerProcessor::LoadBallShadowTexture | confirmed | Fastcall (this in ECX). A static byte at 0x01BE9C34 guards a one-time setup that registers an atexit callback. Each call formats the string bbe_soccerball_shad… |
| `00b7bce0` | SoccerProcessor::CreateLocalPlayer | confirmed | Thiscall with two stack arguments (a source object and a flag). When the source is nonzero, it calls vtable +0x10 of the object at source+0x54 to obtain the ke… |
| `00b7be70` | SoccerProcessor::UpdateTimerView | confirmed | Thiscall with a float time and a flag byte. When the flag is zero and the time differs from this+0x120 by at least this+0x124, when the time is at most DAT_017… |
| `00b7c070` | SoccerProcessor::UpdateScoreboard | confirmed | Thiscall with two team scores and a force flag. When score 0 differs from this+0x128 or the flag is set, it pushes the score and a team index through 007071f0 … |
| `00b80080` | SoccerProcessor::Ctor | confirmed | Installs AssetRequester then SoccerProcessor vtables; initializes team fields, strings, status and pickup lists, arrays, flags and timers; assigns bbe_goal_mad… |
| `00b807c0` | SoccerProcessor::Dtor | confirmed | Restores the SoccerProcessor vtable, releases held objects and sound slots, destroys pickup/status lists and string members, releases three referenced objects,… |
| `00b83330` | SoccerProcessor::ScalarDeletingDtor | confirmed | Scalar deleting destructor. Calls the dtor body FUN_00b807c0 on ECX (this), then calls _free on this when bit 0 of the flag argument is set, and returns this. |
| `00b7a0b0` | SoccerProcessor::LoadKeyedFields | likely | Fastcall (this in ECX). Looks up four keyed entries through 00726e30 (keys 0x6DA, 0x6D9, 0x6D8, 0x6D7), passes each entry's C string at +0x24 to 00402ce0, and … |
| `00b7a880` | SoccerProcessor::ApplyTeamColourNames | likely | Thiscall with one stack argument (a player record). When the argument is nonzero and the controller from 00cd82f0 yields an object through its vtable +0x24, it… |
| `00b7aad0` | SoccerProcessor::ApplyStarterEffect | likely | Thiscall with one stack argument. When the argument is nonzero, the count at this+0x5b0 is above 1, and the controller from 00cd82f0 yields an object through i… |
| `00b7a4a0` | — | unknown | Thiscall with seven stack arguments. Returns when the value at FUN_00769710()+0x38 is null or this+0x214 is null. When param_8 is nonnull and its +0x2b4 is zer… |

**SoeUtil::List<SoccerPickup,-1>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b75d30` | ListSoccerPickup::AllocNode | confirmed | Vtable slot 2 of the pickup list's allocator interface: allocates through the current thread memory manager's vtable slot +4 (via FUN_006fb9c0/FUN_006fb860), f… |
| `00b75d70` | ListSoccerPickup::FreeNode | confirmed | Vtable slot 3 of the pickup list's allocator interface: frees a node by calling FUN_00401c00(node, 4) — FreeMemory with alignment 4 (18-byte body, no other wor… |

**SoeUtil::List<SoccerPlayerStatus,-1>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b75c30` | ListSoccerPlayerStatus::AllocNode | confirmed | Vtable slot 2 of the list's allocator interface: takes the current thread's memory manager (FUN_006fb9c0 then FUN_006fb860) and allocates through its vtable sl… |
| `00b75c70` | ListSoccerPlayerStatus::FreeNode | confirmed | Vtable slot 3 of the list's allocator interface: frees a node by calling FUN_00401c00(node, 8) — the FreeMemory route with alignment 8 (18-byte body, no other … |

**(no class)** (75)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b73d40` | GetBallForTeamIndex | confirmed | Getter: returns the object at this+0x220+index*4 for index <= 1 (team 0/1) and 0 for any larger index; raw bytes 8b 44 24 04 / 83 f8 01 / 77 0a / 8b 84 81 20 0… |
| `00b73e00` | GetStateScriptName | confirmed | Table getter: loads the index from this+0x5dc and returns the second dword of the 8-byte table entry at 0x01b5e520 + index*8, i.e. the string pointer (bytes 8b… |
| `00b75660` | WorldToFieldNormalized | confirmed | Maps a world-space point into the soccer field's normalized space: out.x = ((in.x - origin.x(+0x5b4)) / scale.x(+0x5cc)) * 2 - 1, out.y = 0, out.z = ((in.z - o… |
| `00b7e2d0` | SetSoccerProcessorActive | confirmed | Stores the requested byte at +0x5d8 and returns 0 if unchanged; on enable it initializes soccer resources, sounds, camera, and UI, while on disable it releases… |
| `00b7e910` | UpdateSoccerProcessorView | confirmed | Updates two scoreboard values when they differ from cached fields, refreshes four UI element positions, invokes the arrow-entry and selected-object updates, up… |
| `00cddf60` | LoadSoccerBallEffectDefinitions | confirmed | Looks up MG_Soccer_Ball_Bounce and MG_Soccer_Goalie_Block, retains each available definition, creates its effect handle, and stores both definition/handle pair… |
| `00b73b20` | ReleaseSoccerSoundSlots | likely | For each of the five definition slots at +0x648, +0x650, +0x658, +0x660 and +0x668: if the slot is non-null, releases the paired runtime handle (at +0x64c/+0x6… |
| `00b73c50` | AllSoundDefsSlot3True | likely | Calls vtable slot 3 (offset +0xc) on each of the five objects stored at +0x648, +0x650, +0x658, +0x660 and +0x668, short-circuiting on the first false result; … |
| `00b73cb0` | CallSoundHandlesWithFloat | likely | For each non-null runtime handle at +0x64c, +0x654, +0x65c, +0x664 and +0x66c, calls FUN_0078fcc0 with param_2 converted to float; null slots are skipped. |
| `00b73d70` | SetFlag5e6 | likely | One-byte field setter: writes 1 to this+0x5e6 and returns; raw bytes c6 81 e6 05 00 00 01 c3 (mov byte ptr [ecx+0x5e6],1; ret) with no other work. |
| `00b73dd0` | GetControllerStateOrThree | likely | Getter: when the object at this+0x21c is non-null returns its field at +0x18, otherwise returns the constant 3 (bytes 8b 81 1c 02 00 00 / 85 c0 / 74 04 / 8b 40… |
| `00b73df0` | GetStateScriptId | likely | Table getter: loads the index from this+0x5dc and returns the first dword of the 8-byte table entry at 0x01b5e520 + index*8 (bytes 8b 81 dc 05 00 00 / 8b 04 c5… |
| `00b73fb0` | ReleaseTeamObjectPair | likely | Releases a team-slot record: if +0x14 is set, calls that object through vtable slot +0xc with argument 1 and clears the field. If +0x10 is set, unregisters it … |
| `00b74020` | ReleaseBothTeamObjects | likely | Calls FUN_00b73fb0 on the team-0 slot record (this+0x618) and then on the team-1 slot record (this+0x630); no other work (bytes 56 / 8b f1 / 8d 86 18 06 00 00 … |
| `00b74290` | Thunk00cd6a30WithThis | likely | EH-framed forwarder: calls FUN_00cd6a30 with the incoming this pointer and no other arguments, then returns; the call is the function's only work (bytes show a… |
| `00b744b0` | ForwardThreeDerefs | likely | Thin adapter: calls FUN_00cd6a70 with the this pointer plus the three dereferenced arguments (*param_2, *param_3 and the byte read from param_4), then returns … |
| `00b75750` | FieldNormalizedToWorldCopy | likely | Identical body to FUN_00b756d0 except that it does not write the w component: out.x and out.z are computed from scale.x(+0x5cc)/scale.z(+0x5d4), DAT_017649d4 a… |
| `00b757c0` | GetField30c | likely | Getter (7 bytes): returns *(this+0x30c); bytes 8b 81 0c 03 00 00 c3 (mov eax,[ecx+0x30c]; ret), no other work. |
| `00b757d0` | ListNthFrom304 | likely | Walks the singly linked list rooted at this+0x304 (next pointer at node+0x40) param_2 times and returns the node reached, or 0 when the index runs past the tai… |
| `00b75ac0` | CallSlot6cOnMatchingEntities | likely | For each of the this+0x30c tracked entities, walks the list at +0x304 skipping a running index, dereferences the node's first pointer and, when that object's +… |
| `00b75c90` | InitEightNodePool | likely | Zeroes param_1[0], param_1[1] and the byte at +0x10, then aligns the node base at (param_1+0x18) to eight bytes and links eight 0x48-byte nodes in a backward c… |
| `00b76050` | FindTrackedObjectByPair | likely | Walks the list at this+0x304 (next at +0x40) and returns the first field of the first node whose +8/+0xc pair equals (param_2, param_3), or 0 when none matches. |
| `00b76090` | ReleaseHandlesViaManager | likely | Obtains the client's resource manager (*(*(this+0x214)+0xe8)+4), releases the four handles at +0x5f8, +0x5fc, +0x600 and +0x604 through its vtable slot +2 (off… |
| `00b76180` | FindTrackedNodeByPair | likely | Walks the list at this+0x304 (next at +0x40) and returns the first node whose +8/+0xc pair equals (param_2, param_3), or 0; unlike FUN_00b76050 it returns the … |
| `00b761b0` | AdvanceStateAnimation | likely | Increments the state index at +0x5dc when the next row of the state-script table (DAT_01b5e528 + index*8) is not -1, then for every entity in the list at +0x30… |
| `00b76210` | ApplyPreviousAnimationState | likely | Decrements the state index at +0x5dc when it is above zero and then, for every entity in the list at +0x304, calls its vtable slot +0x54 with (script id from D… |
| `00b762f0` | FindPickupByField4 | likely | Walks the list rooted at this+0x60c (next pointer at node+0x20) and returns the first node whose +4 field equals param_2, or 0. |
| `00b76b50` | FreeBinaryTree | likely | Recursively frees a binary tree node: frees the children at +8 and +0xc first and then the node itself with FUN_00401c00(node, param_2), where param_2 is the a… |
| `00b76f60` | InitTwoStringArrays | likely | Constructor of the soccer field object: runs _eh_vector_constructor_iterator_ over two 2-element arrays of 0x10-byte string items (at +0xc and +0x3c), then ini… |
| `00b77030` | DtorTwoStringArrays | likely | Destructor: destroys the two 2-element 0x10-byte string arrays with _eh_vector_destructor_iterator_ (element destructor FUN_00401e80), the one at +0x3c first a… |
| `00b771e0` | SetLocalPlayerFlagC0 | likely | Reads the local player's 8-byte guid through the client interface (*(this+0x214)+200, vtable slot +0x10), looks the matching tracked object up with FUN_00b7605… |
| `00b77a70` | TreeClearAndFree | likely | Frees the binary tree rooted at *param_1: recursively frees both children via FUN_00b76b50 with the caller's alignment then the root with FUN_00401c00, and cle… |
| `00b77eb0` | TreeClearAndFree8 | likely | Variant of FUN_00b77a70 with alignment 8: frees the binary tree at *param_1 (children at +8/+0xc via FUN_00b76b50, root via FUN_00401c00(node,8)) and clears pa… |
| `00b7ac20` | ListUnlinkAndDestroyNode | likely | Thiscall (list in ECX) with one node argument. Unlinks the node by its prev at +0x44 and next at +0x40, updating the list head at +4, the tail at +8 and the co… |
| `00cd6af0` | ComposeAndApplyObjectTransform | likely | Resolves the active object, sets two status bits when clear, composes a 4x4 transform, and applies the resulting matrix through the object interface. |
| `00cd70b0` | SetIndexedSoccerActionFlag | likely | Calls a count getter, checks the requested index against that count, then sets the indexed flag at this+0x254 when the index is in range. |
| `00cd7900` | SoccerStateName | likely | Maps soccer state codes to strings such as InitializeGame, StartGame, Halftime, EndGame, PlayingSoccer, and Winner. |
| `00cd80c0` | SetSoccerVectorFields | likely | Stores supplied values into fields at offsets +0x144, +0x14c, and +0x2b0. |
| `00cd80f0` | MapSoccerActionStateCode | likely | Maps selected input codes in the 0x17d7 range through a virtual mode check and returns adjusted codes; four listed codes map to zero and other inputs pass thro… |
| `00cd8470` | ProcessResourceEntryOptionallyTwice | likely | When the supplied resource pointer is nonzero, resolves the global object at +0x16c and calls 007e7f00 on the entry at +4; repeats the call when the global obj… |
| `00cd84c0` | RunSixResourceEntries | likely | Calls 007e7f00 for the six resource entries stored across offsets +0x388 through +0x3b0. |
| `00cd92d0` | GetOrCreateMappedObject | likely | Selects an indexed field through 00cd7bb0, checks the associated manager/key map, and creates the object with 00902f50 when absent. |
| `00cd9370` | ProcessSelectedResourceEntry | likely | Selects one field through 00cd7bb0 and passes the selected value to 007e7f00 for processing. |
| `00cd93c0` | GetOrCreateKeyedObject | likely | Checks the keyed/active state and calls 00903180 to create the missing entry when its guards pass. |
| `00cd96a0` | BlendAndScaleVectorAt50 | likely | Accepts a vector, optionally blends it with the four values at +0x50 through +0x5c using weight +0x2b0, scales it when its norm exceeds the threshold, and stor… |
| `00cd97d0` | BlendAndScaleVectorAt30 | likely | Accepts a vector, optionally blends it with the four values at +0x30 through +0x3c using weight +0x2b0, scales it when its norm exceeds the threshold, and stor… |
| `00cd9950` | TransformAndLookupMappedObject | likely | Builds a four-value local transform from supplied inputs, transforms it through 0073a2b0, and passes the result into 00cd92d0. |
| `00cd9ad0` | InitializeSoccerMappedState | likely | Stores the global value in this+0x2ac and calls 00cd93c0 with the field at +0x37c. |
| `00cda6a0` | CopyStructuredUpdateRecord | likely | Copies scalar fields and runs two exception-aware array copy constructors, each with stride 0x10 and count two, then copies the remaining fields and returns th… |
| `00cde480` | InitializeObjectAndEffectState | likely | Initializes fields from its arguments, resolves a resource through the supplied object, copies transform values, updates a referenced component, loads soccer-b… |
| `0107bac0` | SetFieldB0IfNonZero | likely | Thiscall with one stack argument (ret 4): when the argument is nonzero stores it at this+0xb0 and clears the float at this+0x40 to 0.0; otherwise does nothing. |
| `0107bb00` | Wrapper_0107bb00 | likely | Loads ECX from the dword at [ECX+0x48] and jumps to 00fe2700 (tail jump; no frame). |
| `0107bb10` | SetDwords4At60 | likely | Thiscall with one pointer argument to four dwords: when the handle at +0x48 is non-null calls 00fe08a0(argument); then copies the four dwords to this+0x60..+0x… |
| `0107bb50` | Wrapper_0107bb50 | likely | Loads ECX from the dword at [ECX+0x48] and jumps to 00fe0810 (tail jump; no frame). |
| `0107be80` | SoccerBallBase::ClampSetAt74Ratio78 | likely | Thiscall with two float arguments and no return value. Clamps the first to [0, DAT_017ed4dc] and stores it at this+0x74; when the second is positive, raises it… |
| `00afb290` | StoreFloat4Pair188AndDelta | speculative | Thiscall: from four floats at the argument, stores p0 at this+0x188, zero at +0x18c, p1 at +0x190, p2 at +0x194, zero at +0x198, p3 at +0x19c, and the differen… |
| `00b73d60` | SetFlag5e5 | speculative | One-byte field setter: writes 1 to this+0x5e5 and returns; raw bytes c6 81 e5 05 00 00 01 c3 (mov byte ptr [ecx+0x5e5],1; ret) with no other work. |
| `00b73e10` | GetOwnerField10 | speculative | Null-safe getter: returns *(param_1+0x10) when param_1 is non-null and 0 otherwise (bytes 8b 44 24 04 / 85 c0 / 74 06 / 8b 40 10 / c2 04 00 / 33 c0 / c2 04 00). |
| `00b73e70` | StartTeamHandle | speculative | For team index 0/1 (slot at +0x618 or +0x630): when the slot has no live object at +0x14 but has a source object at +0x10 whose script name (byte at +0x64) is … |
| `00b75a60` | ClearTeamOverlayHandles | speculative | For each team record (+0x62c for team 0, +0x644 for team 1): when the pointer is set and its word at +0x1c is zero, releases it through vtable slot +0xc with a… |
| `00b75d90` | ListAppendNode | speculative | Appends an entry to the list rooted at param_1: allocates a node through the list's vtable slot 2, constructs it with FUN_00b744b0 (which forwards the three de… |
| `00b75e30` | SoccerNodeScalarDtor | speculative | Scalar deleting destructor: runs the node destructor FUN_00cd6a30 (with the EH cookie guard) and frees the object with _free when the flag byte's low bit is se… |
| `00b76150` | FindTrackedByField34 | speculative | Walks the list at this+0x304 (next at +0x40) and returns the first node whose +0x34 field equals param_2, or 0. |
| `00b76a90` | ListRemoveAndFreeNode | speculative | Unlinks the node param_2 from the doubly linked list param_1 (patches the neighbours' +0x20/+0x24 links and head/tail when the node was first/last), decrements… |
| `00b77a00` | ListRemoveAll | speculative | Repeatedly removes and frees the head node of the list at param_1 (node pointer at param_1+4) through FUN_00b76a90 until the list is empty. |
| `00b75570` | — | unknown | Jump thunk (11 bytes): loads ecx from this+0x20c and tail-jumps to FUN_00cd70b0 (8b 89 0c 02 00 00 / e9 35 1b 16 00); it performs no other work and never retur… |
| `00b756d0` | — | unknown | Inverse mapping of FUN_00b75660: out.x = scale.x(+0x5cc) * DAT_017649d4 * (in.x + 1) + origin.x(+0x5b4), out.y = 0, out.z = scale.z(+0x5d4) * DAT_017649d4 * (i… |
| `00b76320` | — | unknown | Builds a 4-float vector (param_3, param_4, param_5, 1.0) and passes it through FUN_0073a2b0 into a 48-byte block, copies the caller's 4-float vector param_2 in… |
| `00b763b0` | — | unknown | Builds a 76-byte structure with FUN_00738c30 (three constant operands DAT_01bc2660/DAT_01bc2670/DAT_01bc2680 plus param_3) and, for team 0/1 (slot +0x618 or +0… |
| `00b77480` | — | unknown | Scans the tracked-player list at +0x304 for an entity in the same group (matching +0x18) whose state +0x1c is not 3, picks the best candidate by the dot-produc… |
| `00b83710` | — | unknown | Thiscall with one argument (dt). Does nothing unless the byte at +0x5d8 is set. When the state word +0x1e8 is above 5, it walks the list at +0x304 and, for eac… |
| `00b839e0` | — | unknown | Fastcall with this in ECX. Sets the byte at +0x5e3 to 1. When the state word at +0x1e8 is 8, it sets it to 9, calls FUN_00b752a0 (SoccerPacketUpdateGameState::… |
| `00b83ae0` | — | unknown | Fastcall with this in ECX. Constructs SoccerPacketAcquirePickUp (FUN_00b74ee0, sub 10) on a stack temporary (ECX = its address). It then stores zero into a sta… |
| `00cd79c0` | — | unknown | Divides the value at this+0x200 by this+0xcc when the numerator and denominator conditions pass; otherwise returns the fallback constant. |
| `00cd8ec0` | — | unknown | Resolves and returns the mapped record associated with the supplied object state. |

### `ui.soccer`

**SoccerPlayer** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cdac00` | SoccerPlayer::UpdateOverheadText | likely | Looks up the associated UI object, updates overhead-element fields through indexed interface calls, and constructs an OverHeadTextElement when the guarded elem… |
| `00cdae70` | SoccerPlayer::UpdateOverlayElementState | likely | When the associated element passes its guard, updates indexed interface fields from this player's state and calls the shared element-update helper. |

**SoccerPlayerActionsDataSource** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cd7690` | SoccerPlayerActionsDataSource::~SoccerPlayerActionsDataSour… | confirmed | Rewrites the player actions data-source vtable and destroys its embedded GuiUserDataTable base member. |
| `00cd78e0` | SoccerPlayerActionsDataSource::DeletingDestructor | speculative | Calls the player actions data-source destructor, checks flag bit zero, conditionally frees the object, and returns its address. |
| `00cd7840` | — | unknown | Constructs the embedded GUI user-data table, stores the SoccerPlayerActions identifier, and clears the eight action cells at offset 0x254. |

**SoccerPlayerStatsDataSource** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cd7620` | SoccerPlayerStatsDataSource::~SoccerPlayerStatsDataSource | confirmed | Rewrites the player statistics data-source vtable and destroys its embedded GuiUserDataTable base member. |
| `00cd77a0` | SoccerPlayerStatsDataSource::SoccerPlayerStatsDataSource | confirmed | Constructs the embedded GUI user-data table and stores the SoccerPlayerStats data-source identifier. |
| `00cd7820` | SoccerPlayerStatsDataSource::DeletingDestructor | speculative | Calls the player statistics data-source destructor, checks flag bit zero, conditionally frees the object, and returns its address. |

**SoccerTeamStatsDataSource** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cd75b0` | SoccerTeamStatsDataSource::~SoccerTeamStatsDataSource | confirmed | Rewrites the team statistics data-source vtable and destroys its embedded GuiUserDataTable base member. |
| `00cd7700` | SoccerTeamStatsDataSource::SoccerTeamStatsDataSource | confirmed | Constructs the embedded GUI user-data table and stores the SoccerTeamStats data-source identifier. |
| `00cd7780` | SoccerTeamStatsDataSource::DeletingDestructor | speculative | Calls the team statistics data-source destructor, checks flag bit zero, conditionally frees the object, and returns its address. |

**(no class)** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b7c520` | UpdateSoccerPossessionUi | confirmed | Reads previous flags at +0x134/+0x135, recomputes them from eligible entries in the list at +0x304, stores the new flags, and invokes SoccerScores:SetPossessio… |
| `00b7c6b0` | UpdateSoccerArrows | confirmed | Computes a screen-space arrow target from the object at +0x220, tests its projected position against viewport edges, and invokes SoccerArrows show, hide, posit… |
| `00b7d210` | UpdateSoccerArrowEntries | confirmed | Walks linked entries at +0x304, projects their associated positions, compares player identifiers against stored identifier pairs, and issues SoccerArrows show,… |
| `00b7de60` | ApplySoccerUiState | confirmed | Compares the current state at +0x1e8 with a saved state and dispatches soccer Lua callbacks for wait, ready, halftime, goal, sudden-death, timer, and results s… |
| `00b77220` | PositionUiElement | likely | Looks the target element up by id through the client interface slot +0x10; when it exists and param_3 is above the zero constant it enables the element (FUN_00… |

## Source shards

[[S0330|S0330]] (10), [[S0331 00afb7a0-00b005c0|S0331]] (2), [[S0353 00b71490-00b77eb0|S0353]] (50), [[S0354|S0354]] (9), [[S0355 00b7c520-00b807c0|S0355]] (8), [[S0356 00b80ac0-00b84ce0|S0356]] (4), [[S0419 00cd5e90-00cdb1a0|S0419]] (64), [[S0420 00cdb370-00ce0140|S0420]] (16), [[S0577|S0577]] (12), [[S0578|S0578]] (10), [[S0579|S0579]] (21)
