# Class: PacketSetAutomount

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b03810` | `PacketSetAutomount::PacketSetAutomount` | ✅ | ctor | Constructor: calls the MountBasePacket family constructor FUN_0089f9c0(9) (which writes the family opcode 0xa8 and sub-opcode 9) and installs PacketSetAutomount::vftable; no payload field is initialised. |
| `00b03f20` | `PacketSetAutomount::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor at vtable slot 0: writes BasePacket::vftable into *param_1, calls _free(param_1) when the low bit of param_2 is set and returns the object pointer. |
