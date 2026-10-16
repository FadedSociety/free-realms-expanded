# Class: ControllerSingleRotation

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a6eda0` | `ControllerSingleRotation::ControllerSingleRotation ⚠downgrade` | 🟢 | ctor | Constructor: calls the base constructor 0094f6c0, zeroes the float at +0xe0 (rotation angle), stores param_6 at +0xe4 (radius) and param_7 at +0xe8 (height offset), stores param_5 at +0xec (period) and replaces it with … |
| `00a6ee70` | `ControllerSingleRotation::~ControllerSingleRotation` | ✅ | dtor | Destructor body: stores the ControllerSingleRotation vtable at this, then calls the base destructor 0094fab0. It does not free this. |
| `00a6eec0` | `ControllerSingleRotation::ConsumeFullTurn` | 🟢 | virtual | Virtual slot 35 (+0x8c), 34 bytes: when the float at +0xe0 (angle) is greater than the constant DAT_017eb268 (bits 0x40c90fdb, which is 2*pi as a float) it stores 0 in +0xe0 and returns 1; otherwise returns 0. |
| `00a6eef0` | `ControllerSingleRotation::ForwardFloatToSharedSlot70 ?` | 🟡 | virtual | Virtual slot 28 (+0x70), 29 bytes: gets an object from 0094c6f0 and, when it is non-null, calls that object's virtual slot 28 (+0x70) with the float argument. Returns nothing. |
| `00a6ef10` | `ControllerSingleRotation::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor body: stores the ControllerSingleRotation vtable, calls the base destructor 0094fab0, then frees this through _free when bit 0 of the flag argument is set; returns this. |
| `00a6ef70` | `— ⚠reject` | ⚪ | virtual | Virtual slot 51 (+0xcc). Gets the centre object from 0094cf60 and reads its position through the virtual at +0x1c. Advances the angle at +0xe0 by (2*pi / period at +0xec) times param_2. Recomputes the position from the … |
