# Class: PetTrickFavoriteSetPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b51290` | `PetTrickFavoriteSetPacket::PetTrickFavoriteSetPacket` | ✅ | ctor | Constructor: calls the pet base constructor 00b50670 with subtype 0xe (14), stores PetTrickFavoriteSetPacket::vftable and zeroes the three dwords at +0xc/+0x10/+0x14. |
| `00b512f0` | `PetTrickFavoriteSetPacket::~PetTrickFavoriteSetPacket` | ✅ | dtor | Destructor: stores PetTrickFavoriteSetPacket::vftable then BasePacket::vftable at *this; the pet dispatcher 00b5ce60, the sender 00b5f670 and two unwind funclets call it. |
| `00b51340` | `PetTrickFavoriteSetPacket::ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor: resets the vtable pointer at *this to PetTrickFavoriteSetPacket::vftable then BasePacket::vftable and calls _free(param_1) when bit 0 of the flag argument is set. |
