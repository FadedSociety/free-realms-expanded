# Class: CommandPacketShowRecipeWindow

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a987a0` | `CommandPacketShowRecipeWindow::CommandPacketShowRecipeWindow` | ✅ | ctor | Calls 008a2820 with sub-opcode 0x19 and zero extra word, then stores CommandPacketShowRecipeWindow::vftable and zeroes the words at +0x10 and +0x14. |
| `00a98810` | `CommandPacketShowRecipeWindow::~CommandPacketShowRecipeWindow` | ✅ | dtor | Stores CommandPacketShowRecipeWindow::vftable, then BaseCommandPacket::vftable, then BasePacket::vftable into the object. Called from EH unwind funclet 015c1380 and from 00aa0920. |
| `00a98870` | `CommandPacketShowRecipeWindow::ScalarDeletingDtor` | ✅ | dtor | Stores the CommandPacketShowRecipeWindow, BaseCommandPacket and BasePacket vtables in order, then if the low bit of the flag byte is set calls _free on the object; returns the object pointer. |
