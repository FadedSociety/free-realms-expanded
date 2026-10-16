# Class: CommandPacketSetProfileByItemDefinitionId

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a2b20` | `CommandPacketSetProfileByItemDefinitionId::CommandPacketSetProfileByItemDefinit…` | ✅ | ctor | Constructor: calls FUN_008a2820(0x24,0) so the packet is command opcode 26 with sub-opcode 36, then installs CommandPacketSetProfileByItemDefinitionId::vftable; the sender 0092ef50 uses it when a profile-changing item i… |
| `008a2b80` | `CommandPacketSetProfileByItemDefinitionId::~CommandPacketSetProfileByItemDefini…` | 🟢 | dtor | Destructor body: restores the derived vtable, then BaseCommandPacket::vftable and BasePacket::vftable. |
| `008a2be0` | `CommandPacketSetProfileByItemDefinitionId::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor: restores the SetProfileByItemDefinitionId, command and BasePacket vtables in an SEH frame, then calls _free(this) when param_2 bit 0 is set. |
