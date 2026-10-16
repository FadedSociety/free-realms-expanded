# Class: PetDecisionPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b50730` | `PetDecisionPacket::PetDecisionPacket` | ✅ | ctor | Constructor: calls the pet base constructor 00b50670 with subtype 1, then stores PetDecisionPacket::vftable at *this inside an SEH frame. |
| `00b50790` | `PetDecisionPacket::~PetDecisionPacket` | 🟢 | dtor | Destructor: resets the vtable pointer at *this to BasePacket::vftable and returns; the class has only scalar members, so no other teardown is emitted. |
| `00b52e10` | `PetDecisionPacket::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor: resets the vtable pointer at *this to BasePacket::vftable and calls _free(param_1) when bit 0 of the flag argument is set; returns the object pointer. The exe RTTI records this address as slo… |
