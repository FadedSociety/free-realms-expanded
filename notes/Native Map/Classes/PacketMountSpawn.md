# Class: PacketMountSpawn

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b038c0` | `— ⚠reject` | ⚪ | ctor | Constructor: calls FUN_0089f9c0(6) (family opcode 0xa8, sub-opcode 6), installs PacketMountSpawn::vftable and initialises the payload dword at +0xc to 0xffffffff (the unset mount number). |
| `00b03f80` | `PacketMountSpawn::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor at vtable slot 0: writes BasePacket::vftable into *param_1, calls _free(param_1) when the low bit of param_2 is set and returns the object pointer. |
