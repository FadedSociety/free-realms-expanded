# Class: ExecuteScriptPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a8b710` | `ExecuteScriptPacket::Construct` | ✅ | ctor | Constructs an ExecuteScriptPacket: calls the BaseUiPacket constructor 00a86640 with sub-opcode 7, stores ExecuteScriptPacket::vftable at *this, an empty IString<char> at +0xc (buffer DAT_01be7a0c, count 0), zeroes +0x14… |
| `00a8d4e0` | `ExecuteScriptPacketDeletingDestructor` | ✅ | dtor | Calls the packet destructor body, frees the object only when flag bit 0 is set, and otherwise returns the original object pointer. |
