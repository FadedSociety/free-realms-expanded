# Class: PetSummonByItemIdPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b50940` | `PetSummonByItemIdPacket::PetSummonByItemIdPacket` | ✅ | ctor | Constructor: calls the pet base constructor 00b50670 with subtype 0x20 (32), then stores PetSummonByItemIdPacket::vftable at *this. |
| `00b509a0` | `PetSummonByItemIdPacket::~PetSummonByItemIdPacket` | 🟢 | dtor | Destructor: resets the vtable pointer at *this to BasePacket::vftable and returns. |
| `00b52f30` | `PetSummonByItemIdPacket::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor: resets the vtable pointer at *this to BasePacket::vftable and calls _free(param_1) when bit 0 of the flag argument is set; returns the object pointer. The exe RTTI records this address as slo… |
