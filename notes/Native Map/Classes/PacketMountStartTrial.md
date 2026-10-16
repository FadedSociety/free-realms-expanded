# Class: PacketMountStartTrial

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b03a20` | `PacketMountStartTrial::PacketMountStartTrial` | ✅ | ctor | Constructor: calls FUN_0089f9c0(10) (family opcode 0xa8, sub-opcode 10) and installs PacketMountStartTrial::vftable; no payload field is initialised. |
| `00b04040` | `PacketMountStartTrial::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor at vtable slot 0: writes BasePacket::vftable into *param_1, calls _free(param_1) when the low bit of param_2 is set and returns the object pointer. |
