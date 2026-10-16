# Class: PetUtilityPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b514e0` | `PetUtilityPacket::PetUtilityPacket` | ✅ | ctor | Constructor: calls the pet base constructor 00b50670 with subtype 0x10 (16), stores PetUtilityPacket::vftable, zeroes the byte at +0xc and the dwords at +0x10/+0x14, constructs an embedded member through 0101cd40 and ze… |
| `00b51550` | `PetUtilityPacket::~PetUtilityPacket` | ✅ | dtor | Destructor: stores PetUtilityPacket::vftable then BasePacket::vftable at *this; three unwind funclets and the senders 00b5ce60/00c077a0 call it. |
| `00b515a0` | `PetUtilityPacket::ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor: resets the vtable pointer at *this to PetUtilityPacket::vftable then BasePacket::vftable and calls _free(param_1) when bit 0 of the flag argument is set. |
