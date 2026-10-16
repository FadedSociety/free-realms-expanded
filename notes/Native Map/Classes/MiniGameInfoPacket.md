# Class: MiniGameInfoPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009b9c70` | `MiniGameInfoPacket::MiniGameInfoPacket` | ✅ | ctor | Constructs the family-39 sub-opcode 16 (0x10) info packet: calls FUN_008a4420(0x10,0,-1,-1), installs MiniGameInfoPacket::vftable, initialises the embedded state record at +0x18 through FUN_009b9b30 and zeroes +0x298. |
| `009b9cf0` | `MiniGameInfoPacket::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor for the 39/16 info packet: calls the packet destructor body 009b0900. Then frees the object through _free when the low bit of param_2 is set (test byte [esp+8],1) and returns the pointer in EA… |
