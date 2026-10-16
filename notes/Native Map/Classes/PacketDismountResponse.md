# Class: PacketDismountResponse

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b046a0` | `PacketDismountResponse::PacketDismountResponse ⚠downgrade` | 🟢 | ctor | Constructor: calls FUN_0089f9c0(4) (family opcode 0xa8, sub-opcode 4), installs PacketDismountResponse::vftable, stores the invalid-guid pair DAT_01be9138/DAT_01be913c at +0x10/+0x14 and zeroes the dword at +0x18. |
| `00b04710` | `PacketDismountResponse::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor at vtable slot 0: writes BasePacket::vftable into *param_1, calls _free(param_1) when the low bit of param_2 is set and returns the object pointer. |
