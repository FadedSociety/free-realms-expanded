# Class: PetResponsePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c06460` | `— ⚠reject` | ⚪ | ctor | Pet-family packet constructor for subtype 2: calls the shared pet base-packet constructor FUN_00b50670(2) (which writes base opcode 0x35 and the subtype) and installs PetResponsePacket::vftable into the new object. |
| `00c06ae0` | `PetResponsePacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor, slot 0 of PetResponsePacket::vftable: rewrites the object's vtable to BasePacket::vftable and calls _free when bit 0 of the flag byte argument is set. |
