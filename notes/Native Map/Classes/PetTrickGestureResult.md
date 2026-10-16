# Class: PetTrickGestureResult

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b50c20` | `PetTrickGestureResult::PetTrickGestureResult` | ✅ | ctor | Constructor: calls the pet base constructor 00b50670 with subtype 8, then stores PetTrickGestureResult::vftable at *this. |
| `00b50c80` | `PetTrickGestureResult::~PetTrickGestureResult` | ✅ | dtor | Destructor: stores PetTrickGestureResult::vftable then BasePacket::vftable at *this; one unwind funclet calls it. |
| `00b50cd0` | `PetTrickGestureResult::ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor: resets the vtable pointer at *this to PetTrickGestureResult::vftable then BasePacket::vftable and calls _free(param_1) when bit 0 of the flag argument is set. |
