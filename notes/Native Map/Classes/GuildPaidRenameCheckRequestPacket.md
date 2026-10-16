# Class: GuildPaidRenameCheckRequestPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a16c50` | `GuildPaidRenameCheckRequestPacket_Ctor` | ✅ | ctor | Constructor: calls the guild packet base constructor FUN_00a09b70 with sub-opcode 0x10, stores GuildPaidRenameCheckRequestPacket::vftable, copies the two argument dwords to +0x10 and +0x14, initialises the IString membe… |
| `00a16ce0` | `GuildPaidRenameCheckRequestPacket_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0): delegates to the destructor body 00a151d0, which releases the IString member at +0x18, and frees the packet with _free when bit 0 of the flag argument is set. |
