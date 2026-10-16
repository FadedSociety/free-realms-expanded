# Class: CommandPacketStartFlashGame

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a9b850` | `CommandPacketStartFlashGame::CommandPacketStartFlashGame` | ✅ | ctor | Calls 008a2820 with sub-opcode 0xc (12) and zero extra word, stores CommandPacketStartFlashGame::vftable, sets two empty SoeUtil::IString<char> members at +0x10 and +0x20 (vtable, buffer DAT_01be7abc) and zeroes their c… |
| `00a9b8d0` | `CommandPacketStartFlashGame::~CommandPacketStartFlashGame` | 🟢 | dtor | Stores the IString<char> vtable at +0x20, releases the string at +0x24 when its count at +0x2c is positive, stores the IString<char> vtable at +0x10, releases the string at +0x14 under the same test, then stores BaseCom… |
| `00a9c250` | `CommandPacketStartFlashGame::ScalarDeletingDtor` | 🟢 | dtor | Calls 00a9b8d0 (the StartFlashGame destructor body) on this, then if the low bit of the flag byte is set calls _free on the object and returns it. |
