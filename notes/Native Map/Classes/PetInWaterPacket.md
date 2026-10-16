# Class: PetInWaterPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b51bd0` | `PetInWaterPacket::PetInWaterPacket` | ✅ | ctor | Constructor: calls the pet base constructor 00b50670 with subtype 0x1c (28), then stores PetInWaterPacket::vftable at *this. |
| `00b53270` | `PetInWaterPacket::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor: resets the vtable pointer at *this to BasePacket::vftable and calls _free(param_1) when bit 0 of the flag argument is set; returns the object pointer. The exe RTTI records this address as slo… |
