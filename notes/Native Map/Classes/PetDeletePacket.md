# Class: PetDeletePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c06510` | `PetDeletePacket_DtorBody` | ✅ | dtor | Destructor body for PetDeletePacket: rewrites the vtable to PetDeletePacket::vftable, then to BasePacket::vftable, and returns; reached from one MSVC exception-unwind funclet (015f40f0) of the pet delete send path. |
| `00c06560` | `PetDeletePacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor, slot 0 of PetDeletePacket::vftable: rewrites the vtable to PetDeletePacket::vftable then BasePacket::vftable and calls _free on the object when bit 0 of the flag byte argument is set. |
| `00c06b40` | `— ⚠reject` | ⚪ | ctor | Pet-family packet constructor for subtype 17: calls the pet base constructor FUN_00b50670(0x11) (base opcode 0x35, subtype 0x11), installs PetDeletePacket::vftable, and initialises the packet fields at +0xc/+0x10/+0x14 … |
