# Class: BattleMagesPacketCommand

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cd5cc0` | `— ⚠reject` | ⚪ | ctor | Calls the BaseBattleMagesPacket constructor 00b6f2b0 with sub-opcode 1; stores BattleMagesPacketCommand::vftable at this; zeroes +0xc, +0x10, +0x14, +0x18, +0x24 and +0x28; stores SoeUtil::IString<char>::vftable at +0x1… |
| `00cd5d40` | `BattleMagesPacketCommand::~BattleMagesPacketCommand` | ✅ | dtor | Stores SoeUtil::IString<char>::vftable at +0x1c. When +0x28 is positive it atomically decrements the dword at (+0x20)-4; when the previous count was 1 or not positive it calls slot 2 of the IString vtable with that addr… |
| `00cd5dd0` | `BattleMagesPacketCommand::ScalarDeletingDtor` | ✅ | dtor | Virtual slot 0: calls destructor body 00cd5d40 on this, then frees this when bit 0 of the flag argument is set; returns this. |
