# Class: ClientInterpolatedMovementController

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b16b80` | `ClientInterpolatedMovementController::ClientInterpolatedMovementController` | ✅ | ctor | Constructor: calls the base constructor 01077a70 (which runs the MovementController chain and installs InterpolatedMovementController::vftable) and then installs ClientInterpolatedMovementController::vftable into *this. |
| `00b16c40` | `ClientInterpolatedMovementController::Destruct` | 🟢 | dtor | Destructor body: installs ClientInterpolatedMovementController::vftable, then calls the base destructor 01077940, which walks the chain by writing InterpolatedMovementController::vftable and then MovementController::vft… |
| `00b16c90` | `ClientInterpolatedMovementController::ScalarDeletingDtor` | ✅ | dtor | Slot [0] scalar deleting destructor: installs ClientInterpolatedMovementController::vftable, calls the base destructor 01077940 and then frees the object with _free when bit 0 of the flag argument is set. |
| `00cc4410` | `ClientInterpolatedMovementController::LookupTransformForKeyPair` | ✅ | virtual | Initializes the four-dword output from a global vector, calls 00769710 and the two-word hash lookup 00984960, and, when found, copies four dwords returned by node virtual slot +0x1c into the output. |
| `00cc4600` | `— ⚠reject` | ⚪ | virtual | Looks up the controller node using the two dwords at this+0x10 and +0x14; returns if absent, obtains and updates its four-word transform, stores param_4 at node offsets 0x1890..0x189c, stores clamped param_2 deltas at 0… |
