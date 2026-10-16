# Class: PetUtilityFeedPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b51730` | `PetUtilityFeedPacket::PetUtilityFeedPacket` | ✅ | ctor | Constructor: calls the pet base constructor 00b50670 with subtype 0x13 (19), stores PetUtilityFeedPacket::vftable and zeroes the dwords at +0xc/+0x10. |
| `00b51790` | `PetUtilityFeedPacket::~PetUtilityFeedPacket` | ✅ | dtor | Destructor: stores PetUtilityFeedPacket::vftable then BasePacket::vftable at *this; the feed sender 00b5f720 and one unwind funclet call it. |
| `00b517e0` | `PetUtilityFeedPacket::ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor: resets the vtable pointer at *this to PetUtilityFeedPacket::vftable then BasePacket::vftable and calls _free(param_1) when bit 0 of the flag argument is set. |
