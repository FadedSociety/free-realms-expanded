# Class: PetSummonRecallPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b50890` | `PetSummonRecallPacket::PetSummonRecallPacket` | ✅ | ctor | Constructor: calls the pet base constructor 00b50670 with subtype 4, then stores PetSummonRecallPacket::vftable at *this. |
| `00b508f0` | `PetSummonRecallPacket::~PetSummonRecallPacket` | 🟢 | dtor | Destructor: resets the vtable pointer at *this to BasePacket::vftable and returns. |
| `00b52ed0` | `PetSummonRecallPacket::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor: resets the vtable pointer at *this to BasePacket::vftable and calls _free(param_1) when bit 0 of the flag argument is set; returns the object pointer. The exe RTTI records this address as slo… |
