# Class: PetUtilityNotifyPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b51850` | `PetUtilityNotifyPacket::PetUtilityNotifyPacket ⚠downgrade` | 🟢 | ctor | Constructor: calls the pet base constructor 00b50670 with subtype 0x14 (20), stores PetUtilityNotifyPacket::vftable and zeroes the dwords at +0xc/+0x10. |
| `00b518b0` | `PetUtilityNotifyPacket::~PetUtilityNotifyPacket` | ✅ | dtor | Destructor: stores PetUtilityNotifyPacket::vftable then BasePacket::vftable at *this; the pet dispatcher 00b5ce60 and one unwind funclet call it. |
| `00b51900` | `PetUtilityNotifyPacket::ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor: resets the vtable pointer at *this to PetUtilityNotifyPacket::vftable then BasePacket::vftable and calls _free(param_1) when bit 0 of the flag argument is set. |
