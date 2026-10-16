# Class: ShipCombatProcessor

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a4fb70` | `ShipCombatProcessor_VSlot1_ReleaseAndDispatch` | ✅ | virtual | Virtual method at slot 1 of ShipCombatProcessor: when bit 0 of the word at +0x10 is set it clears that bit (and word 0xfffe), releases the member at +4 through FUN_00735a30 when non-null, then dispatches the object's vi… |
| `00a6a4b0` | `— ⚠reject` | ⚪ | method | Per-frame UI driver of the Pirates match, gated on the tutorial flag at +0x301 for movie playback (Pirates.log lines Tutorial movie prepared, hiding previous loading screen. and Couldn acquire rendering mutex to display… |
| `00a6b0c0` | `— ⚠reject` | ⚪ | method | Advances the state machine through earlier unprocessed states before the requested state, logging each transition and setting state bits in +0x304. Entry actions drive waiting, physics, tutorial, countdown, play, game-o… |
| `00a6b5f0` | `— ⚠reject` | ⚪ | method | Drains the fort, chest, projectile and pickup lists at +0x74, +0x84, +0x54 and +0x64; releases objects at +0x3c4 and +0x308 via their vtable delete calls; releases the tutorial movie at +0x3c0; clears configuration/debu… |
| `00a6b8a0` | `— ⚠reject` | ⚪ | ctor | Constructor of the Pirates ship-combat processor: stores ActorHandler::vftable then ShipCombatProcessor::vftable, keeps the owner in param_1[1], zeroes the state fields (+8..+0x1c including the -1 slots), constructs the… |
| `00a6bcb0` | `ShipCombatProcessor::~ShipCombatProcessor` | ✅ | dtor | Destructor: writes the ShipCombatProcessor vtable and calls 00a6b5f0, destroys debug lists, releases the interface at +0x440 when present and the global release gate is set, then frees six array buffers at +0x3cc (Event… |
| `00a6bff0` | `— ⚠reject` | ⚪ | method | Per-frame update of the Pirates processor, gated on the initialised flag at +0x10: runs the script-side update FUN_00ca7480 and the readiness reporter 00a69480, syncs a flag bit at node+0x79 of the +0x84 entity list fro… |
| `00a6cbb0` | `ShipCombatProcessor::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: calls the processor destructor 00a6bcb0 on this; when bit 0 of the flag argument is set it frees this through _free (thunk 00d82463); returns this. |
