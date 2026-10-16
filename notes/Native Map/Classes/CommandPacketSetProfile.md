# Class: CommandPacketSetProfile

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a29f0` | `CommandPacketSetProfile::CommandPacketSetProfile` | ✅ | ctor | Constructor: calls FUN_008a2820(0xd,0) so the packet is command opcode 26 with sub-opcode 13, then installs CommandPacketSetProfile::vftable. |
| `008a2a50` | `CommandPacketSetProfile::~CommandPacketSetProfile` | 🟢 | dtor | Destructor body: restores CommandPacketSetProfile::vftable, then BaseCommandPacket::vftable, then BasePacket::vftable; its only non-self caller is EH funclet 01591d20. |
| `008a2ab0` | `CommandPacketSetProfile::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor: restores the SetProfile, command and BasePacket vtables in an SEH frame, then calls _free(this) when param_2 bit 0 is set. |
