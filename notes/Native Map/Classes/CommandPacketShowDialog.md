# Class: CommandPacketShowDialog

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a99bd0` | `CommandPacketShowDialog::CommandPacketShowDialog` | ✅ | ctor | Calls 008a2820 with sub-opcode 3 and zero extra word, stores CommandPacketShowDialog::vftable, then fills the fields from +0x10 to +0x8c with default constants: sentinel -1 words, default constant blocks from the data a… |
| `00a9c110` | `CommandPacketShowDialog::~CommandPacketShowDialog` | ✅ | dtor | Stores CommandPacketShowDialog::vftable, then calls 00a9bc20 on the intrusive-list member at this+0x20 (the bytes load lea ecx,[esi+0x20] before the call) to unlink and free its nodes, then stores BaseCommandPacket::vft… |
| `00a9c180` | `CommandPacketShowDialog::ScalarDeletingDtor` | 🟢 | dtor | Calls 00a9c110 (the ShowDialog destructor body) on this, then if the low bit of the flag byte is set calls _free on the object and returns it. |
