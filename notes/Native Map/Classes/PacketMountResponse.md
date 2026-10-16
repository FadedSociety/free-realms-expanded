# Class: PacketMountResponse

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b045b0` | `PacketMountResponse::PacketMountResponse` | ✅ | ctor | Constructor: calls FUN_0089f9c0(2) (family opcode 0xa8, sub-opcode 2), installs PacketMountResponse::vftable, stores the invalid-guid pair DAT_01be9138/DAT_01be913c at +0x10/+0x14 and again at +0x18/+0x1c, and zeroes th… |
| `00b04640` | `PacketMountResponse::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor at vtable slot 0: writes BasePacket::vftable into *param_1, calls _free(param_1) when the low bit of param_2 is set and returns the object pointer. |
