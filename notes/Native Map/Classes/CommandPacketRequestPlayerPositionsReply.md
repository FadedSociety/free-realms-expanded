# Class: CommandPacketRequestPlayerPositionsReply

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa1880` | `CommandPacketRequestPlayerPositionsReply::CommandPacketRequestPlayerPositionsRe…` | ✅ | ctor | Constructor: calls the command-family constructor FUN_008a2820 with sub-opcode 0x23 (35) and 0, stores CommandPacketRequestPlayerPositionsReply::vftable at +0 and a SoeUtil::List<CommandPacketRequestPlayerPositionsReply… |
| `00aa18f0` | `CommandPacketRequestPlayerPositionsReply::~CommandPacketRequestPlayerPositionsR…` | ✅ | dtor | Destructor body: rewrites the class vtable, restores the SoeUtil::List vtable at +0x1c and drains the player-position list by repeatedly calling FUN_00a85d60 on the node at +0x20 until it is null, then rewrites the obje… |
| `00aa1980` | `CommandPacketRequestPlayerPositionsReply::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor: calls the destructor body FUN_00aa18f0 and, when bit 0 of the flags argument is set, calls _free on the object; returns the object pointer. |
