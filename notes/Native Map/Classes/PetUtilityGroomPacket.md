# Class: PetUtilityGroomPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b51610` | `PetUtilityGroomPacket::PetUtilityGroomPacket` | ✅ | ctor | Constructor: calls the pet base constructor 00b50670 with subtype 0x12 (18), stores PetUtilityGroomPacket::vftable and zeroes the dwords at +0xc/+0x10. |
| `00b51670` | `PetUtilityGroomPacket::~PetUtilityGroomPacket` | ✅ | dtor | Destructor: stores PetUtilityGroomPacket::vftable then BasePacket::vftable at *this; the groom sender 00b5f8c0 and one unwind funclet call it. |
| `00b516c0` | `PetUtilityGroomPacket::ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor: resets the vtable pointer at *this to PetUtilityGroomPacket::vftable then BasePacket::vftable and calls _free(param_1) when bit 0 of the flag argument is set. |
