# Class: PetExpireTimePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b51970` | `PetExpireTimePacket::~PetExpireTimePacket` | 🟢 | dtor | Destructor: resets the vtable pointer at *this to BasePacket::vftable and returns; no members are released. |
| `00b53090` | `PetExpireTimePacket::PetExpireTimePacket` | ✅ | ctor | Constructor: calls the pet base constructor 00b50670 with subtype 0x17 (23), then stores PetExpireTimePacket::vftable at *this. |
| `00b530f0` | `PetExpireTimePacket::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor: resets the vtable pointer at *this to BasePacket::vftable and calls _free(param_1) when bit 0 of the flag argument is set; returns the object pointer. The exe RTTI records this address as slo… |
