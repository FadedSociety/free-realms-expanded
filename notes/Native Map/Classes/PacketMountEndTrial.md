# Class: PacketMountEndTrial

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b040a0` | `PacketMountEndTrial::PacketMountEndTrial` | ✅ | ctor | Constructor: calls FUN_0089f9c0(0xb) (family opcode 0xa8, sub-opcode 11), installs PacketMountEndTrial::vftable, stores the invalid-guid pair DAT_01be9138/DAT_01be913c at +0x10/+0x14, zeroes +0x18 and the four dwords at… |
| `00b04120` | `PacketMountEndTrial::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor at vtable slot 0: writes BasePacket::vftable into *param_1, calls _free(param_1) when the low bit of param_2 is set and returns the object pointer. |
