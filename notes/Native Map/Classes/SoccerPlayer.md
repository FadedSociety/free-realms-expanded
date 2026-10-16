# Class: SoccerPlayer

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cd7a70` | `SoccerPlayer::UpdateTimedResources` | 🟢 | method | Updates a timed state flag and processes the resource entries held at offsets 0x3d0 and 0x3d8. |
| `00cd7c30` | `SoccerPlayer::ReleaseResourceReferencePairs` | 🟢 | method | For each of twelve resource pairs, invokes the paired object with argument one when present, decrements a positive reference count, and clears the primary handle. |
| `00cd7fd0` | `SoccerPlayer::IsFloatFieldNonpositive` | ✅ | virtual | Returns one when the float at this+0xac is less than or equal to zero, otherwise returns zero. |
| `00cd7ff0` | `SoccerPlayer::GetBallWhenAvailable` | ✅ | virtual | When the controller and its nested ball state are nonzero, calls the shared ball getter and returns its result; otherwise returns zero. |
| `00cd8010` | `SoccerPlayer::ForwardFloatThroughMappedMethod` | ✅ | virtual | Loads the supplied float, places it on the floating-point stack, forwards it to 01073f20, and returns that helper's result. |
| `00cd8020` | `SoccerPlayer::ResetVectorFromField100` | ✅ | virtual | Copies this+0x100 into this+0x30, clears this+0x34 and this+0x3c, then sets this+0x38 to the global constant minus this+0x104. |
| `00cd8050` | `SoccerPlayer::CallTrackedEntityHelperAndReturnArgument` | ✅ | virtual | Calls 00b77480 with the supplied argument, then returns that same argument unchanged. |
| `00cd8080` | `SoccerPlayer::CopyFourFloatDirection` | ✅ | virtual | Copies four 32-bit values from this+0xf0 through this+0xfc into the output buffer. |
| `00cd80b0` | `SoccerPlayer::GetGlobalLookupField` | ✅ | virtual | Calls the shared lookup helper, reads the resulting object's field at +0x90, and returns that value. |
| `00cd8330` | `SoccerPlayer::HasNearbyActiveRecord → SoccerPlayer::HasNearbyActiveBall` | ✅ | virtual | Checks up to three records and returns one when an active record's horizontal squared distance is within the configured threshold; otherwise returns zero. |
| `00cd8640` | `ReleaseSoccerPlayerResources` | 🟢 | method | Processes the six-entry group, then calls the release helper for eleven resource fields held by the SoccerPlayer object. |
| `00cd8700` | `SoccerPlayer::UpdateResourceSlots` | 🟢 | method | Looks up resource slot data, advances its time value through 0078fcc0, and conditionally updates the associated matrix through the nested +0x10 interface. |
| `00cd89b0` | `SoccerPlayer::UpdateMappedResource` | ✅ | virtual | Calls 01071f90, processes the six resource entries, and conditionally invokes slot 0x0c on the object at this+0x400. |
| `00cd89e0` | `SoccerPlayer::UpdateMappedRecordState` | ✅ | virtual | Guards on controller and record state, passes the supplied argument through virtual calls, may call 00b75570, copies transform data, and invokes companion slots +0x30 and +0x34. |
| `00cd8b00` | `SoccerPlayer::WriteMappedTransform` | ✅ | virtual | Copies four transform values into the mapped record and updates the associated companion object. |
| `00cd8be0` | `SoccerPlayer::GetControllerField1E8` | ✅ | virtual | Returns the nested controller object's field at +0x1e8 when the object exists, otherwise returns one. |
| `00cd8c00` | `SoccerPlayer::GetControllerField148` | ✅ | virtual | Returns the nested controller object's field at +0x148 when the object exists, otherwise returns one. |
| `00cd8c20` | `SoccerPlayer::OnStateChanged → SoccerPlayer::SetStateWithDuration` | ✅ | virtual | When the requested state differs from this+0x60, updates the state field, maps action/state codes, rebuilds animation state, and updates duration at +0xac. |
| `00cd8e60` | `SoccerPlayer::RunConditionalUpdateHelpers ⚠downgrade` | 🟢 | virtual | Calls 00b75ac0 when its state guards pass and then invokes 01071a00. |
| `00cd8f30` | `— ⚠reject` | ⚪ | virtual | Resolves the mapped record through 00cd8ec0, then passes the result to 01071af0. |
| `00cd8f50` | `SoccerPlayer::SetHorizontalDirection` | ✅ | virtual | Calls 01074590, checks the squared length of a vector, performs a lookup when the threshold passes, and sends the accepted direction to 0095dae0. |
| `00cd9010` | `SoccerPlayer::GetControllerVector` | ✅ | virtual | When the controller object is present, copies four values from its fields at +0x5b4, +0x5bc, +0x5c0, and +0x5c8; otherwise returns the fallback values. |
| `00cd9130` | `SoccerPlayer::GetMappedValue6E4` | ✅ | virtual | Returns the mapped object's field at +0x6e4 when lookup succeeds, otherwise returns one. |
| `00cd9180` | `SoccerPlayer::GetMappedValue6E8` | ✅ | virtual | Returns the mapped object's field at +0x6e8 when lookup succeeds, otherwise returns one. |
| `00cd91d0` | `SoccerPlayer::GetNestedControllerObject` | ✅ | virtual | Returns the nested controller pointer at +0x30 when available, otherwise returns zero. |
| `00cd9200` | `SoccerPlayer::BuildOrientationMatrix` | ✅ | virtual | Reads angles at +0x610 and +0x618, builds a rotation matrix through 00770e70, and appends the output vector to the resulting matrix. |
| `00cd9570` | `SoccerPlayer::SetActionResource` | ✅ | virtual | Returns early when this+0x600 is set; otherwise stores the supplied values and dispatches action codes 1 through 5 to resource-entry virtual calls. |
| `00cd9a00` | `— ⚠reject` | ⚪ | virtual | Builds a local event context from the controller at +0x14 and calls 010719b0 to emit it. |
| `00cd9af0` | `SoccerPlayer::OnStateChanged` | ✅ | virtual | Handles SoccerPlayer state changes by updating resource entries and performing state-specific pickup/action transitions. |
| `00cd9df0` | `SoccerPlayer::GetTargetDirectedVector` | ✅ | virtual | Scans the roster for another qualifying entity, converts its world position to field space, computes and normalizes a horizontal direction from this player, and returns a fallback vector when no target qualifies. |
| `00cda080` | `SoccerPlayer::InitializeAnimationAssets` | ✅ | method | Looks up twelve MG_Soccer animation/effect asset names and initializes a corresponding referenced object handle for each entry. |
| `00cda220` | `— ⚠reject` | ⚪ | ctor | Initializes the AssetRequester base and SoccerPlayer vtables, stores constructor inputs and the FreeFlyCamera identifier, zeros many fields, assigns local state IDs, acquires references, and initializes soccer assets. |
| `00cdac00` | `SoccerPlayer::UpdateOverheadText` | 🟢 | method | Looks up the associated UI object, updates overhead-element fields through indexed interface calls, and constructs an OverHeadTextElement when the guarded element is absent. |
| `00cdae70` | `SoccerPlayer::UpdateOverlayElementState` | 🟢 | method | When the associated element passes its guard, updates indexed interface fields from this player's state and calls the shared element-update helper. |
| `00cdb1a0` | `SoccerPlayer::~SoccerPlayer` | ✅ | dtor | Restores SoccerPlayer vtables, unregisters controller references, releases resource slots, updates and clears the overlay element, releases paired references and strings, and tears down the AssetRequester base. |
| `00cdb370` | `SoccerPlayer::UpdateDebugDisplay ? ⚠rename` | 🟡 | virtual | Updates a timer and motion-related fields, then conditionally formats animation, server, game, player, ball, charge, and pickup values for display. |
| `00cdbea0` | `SoccerPlayer::ScalarDeletingDestructor` | ✅ | dtor | Calls the SoccerPlayer destructor body, frees this only when flag bit 0 is set, and otherwise returns this. |
| `00cdc4a0` | `SoccerPlayer::SendUpdatePlayerPos` | ✅ | virtual | Builds SoccerPacketUpdatePlayerPos from this object's identifiers and position-related fields, then calls the payload sender when the gateway handle is present. |
| `00cdc5c0` | `SoccerPlayer::SendUpdatePlayerState` | ✅ | virtual | When enabled and this player is in the sendable state, saves the previous state, fills guid/state/auxiliary/vector fields, and submits SoccerPacketUpdatePlayerState. |
| `00cdc710` | `SoccerPlayer::SendFreeBall` | ✅ | virtual | If the soccer context and ball pointer exist, obtains a four-dword value, fills guid and auxiliary fields, and sends SoccerPacketFreeBall when the gateway handle exists. |
| `00cdc810` | `SoccerPlayer::SendUpdatePlayerControls` | ✅ | virtual | Optionally flips the two input floats, compares them and the integer against cached fields, sends SoccerPacketUpdatePlayerControls on change, then forwards all inputs to 01071b10. |
| `00cdc9a0` | `SoccerPlayer::SendSoccerBallUpdate` | ✅ | virtual | Copies a sixteen-dword transform and related ball flags into SoccerPacketUpdateSoccerBall, then submits it when the gateway handle exists. |
