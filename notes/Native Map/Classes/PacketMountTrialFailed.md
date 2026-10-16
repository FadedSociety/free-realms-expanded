# Class: PacketMountTrialFailed

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b03b20` | `PacketMountTrialFailed::PacketMountTrialFailed` | ✅ | ctor | Constructor: calls FUN_0089f9c0(0xd) (family opcode 0xa8, sub-opcode 13) and installs PacketMountTrialFailed::vftable; no payload field is initialised. |
| `00b04180` | `PacketMountTrialFailed::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor at vtable slot 0: writes BasePacket::vftable into *param_1, calls _free(param_1) when the low bit of param_2 is set and returns the object pointer. |
