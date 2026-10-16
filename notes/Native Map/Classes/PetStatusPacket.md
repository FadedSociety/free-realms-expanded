# Class: PetStatusPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b507e0` | `PetStatusPacket::PetStatusPacket` | ✅ | ctor | Constructor: calls the pet base constructor 00b50670 with subtype 3, then stores PetStatusPacket::vftable at *this. |
| `00b50840` | `PetStatusPacket::~PetStatusPacket` | 🟢 | dtor | Destructor: resets the vtable pointer at *this to BasePacket::vftable and returns. |
| `00b52e70` | `PetStatusPacket::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor: resets the vtable pointer at *this to BasePacket::vftable and calls _free(param_1) when bit 0 of the flag argument is set; returns the object pointer. The exe RTTI records this address as slo… |
