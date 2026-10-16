# Class: GroupChangingJobPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c0af40` | `— ⚠reject` | ⚪ | ctor | Group packet constructor for sub-opcode 0xe: calls the group base constructor FUN_008a35c0(0xe) (base opcode 0x28), installs GroupChangingJobPacket::vftable and stores the caller's flag byte at +0xc (the byte that carri… |
| `00c0b0a0` | `GroupChangingJobPacket_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor, slot 0 of GroupChangingJobPacket::vftable: rewrites the vtable to BaseGroupPacket::vftable then BasePacket::vftable and calls _free when bit 0 of the flag byte argument is set. |
