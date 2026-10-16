# Class: PetEquipPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b50e00` | `PetEquipPacket::PetEquipPacket` | ✅ | ctor | Constructor: calls the pet base constructor 00b50670 with subtype 0xb (11), stores PetEquipPacket::vftable, zeroes the byte at +0xc, constructs an embedded member through 0101cd40 and zeroes the dword at +0x14. |
| `00b50e70` | `PetEquipPacket::~PetEquipPacket` | ✅ | dtor | Destructor: stores PetEquipPacket::vftable then BasePacket::vftable at *this; five unwind funclets and the senders 00b5ce60/00c07990 call it. |
| `00b50ec0` | `PetEquipPacket::ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor: resets the vtable pointer at *this to PetEquipPacket::vftable then BasePacket::vftable and calls _free(param_1) when bit 0 of the flag argument is set. |
