# Class: MiniGameFlashCommandAdminPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009af9d0` | `MiniGameFlashCommandAdminPacket_Dtor` | 🟢 | dtor | Restores BasePacket vtable and releases its IString<char> member buffer. |
| `009b3110` | `MiniGameFlashCommandAdminPacketCtor` | ✅ | ctor | Calls the base admin packet constructor with sub-opcode 0x3f, installs MiniGameFlashCommandAdminPacket vtable, initializes an IString member and copies the passed command string. |
| `009b31a0` | `MiniGameFlashCommandAdminPacket::ScalarDeletingDtor` | ✅ | virtual | Calls FUN_009af9d0 to destroy members; when deletion flag bit 0 is set, calls free(param_1), then returns the object pointer otherwise. |
