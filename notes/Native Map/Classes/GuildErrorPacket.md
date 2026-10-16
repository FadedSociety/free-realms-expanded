# Class: GuildErrorPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b19c70` | `GuildErrorPacket::GuildErrorPacket ⚠downgrade` | 🟢 | ctor | Constructor: calls the base guild constructor 00b17220 (which writes opcode 129 and leaves the sub-opcode word at zero), installs GuildErrorPacket::vftable, initializes the embedded IString at +0xc with the shared empty… |
| `00b19ce0` | `— ⚠reject` | ⚪ | dtor | Destructor body of the guild error packet: rewrites the IString vtable at +0xc, releases the string buffer through IString vtable slot 2 when the reference count is positive, then restores BaseGuildPacket::vftable and B… |
| `00b1a4d0` | `GuildErrorPacket::ScalarDeletingDtor` | ✅ | dtor | Slot [0] scalar deleting destructor of GuildErrorPacket::vftable: calls the destructor body 00b19ce0 and frees the object with _free when bit 0 of the flag argument is set. |
| `00b1ad70` | `GuildErrorPacket::ReadBody` | 🟢 | method | Reads the body of a guild error packet: two 16-bit header values through 00b1a810 and then a length-prefixed block skipped through 00894b10, flagging the reader when the block runs past the end; unlike its siblings it r… |
