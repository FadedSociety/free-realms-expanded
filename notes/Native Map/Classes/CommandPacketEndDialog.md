# Class: CommandPacketEndDialog

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009f5760` | `CommandPacketEndDialog::CommandPacketEndDialog` | ✅ | ctor | Constructor: stores 0xffffffff into the EH state slot, installs a SEH frame, calls FUN_008a2820(4,0) (the BaseCommandPacket family constructor: sub-opcode 4, extra word 0), then stores CommandPacketEndDialog::vftable in… |
| `009f57c0` | `CommandPacketEndDialog::DtorBody` | ✅ | dtor | Destructor body without free: stores CommandPacketEndDialog::vftable, then BaseCommandPacket::vftable, then BasePacket::vftable into *this in that order and returns. Called from two EH unwind funclets. |
| `009f5820` | `CommandPacketEndDialog::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor, vtable slot [0] of CommandPacketEndDialog::vftable: stores CommandPacketEndDialog, BaseCommandPacket and BasePacket vtables in order, sets the EH state to -1, and calls _free(this) through th… |
