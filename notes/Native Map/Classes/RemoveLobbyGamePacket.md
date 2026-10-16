# Class: RemoveLobbyGamePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a41f0` | `RemoveLobbyGamePacket::RemoveLobbyGamePacket` | ✅ | ctor | Constructor for lobby sub-opcode 8: calls the opcode-101 family constructor FUN_008a3ce0(8), installs RemoveLobbyGamePacket::vftable; body is wrapped in an MSVC SEH frame that restores ExceptionList. |
| `008a4250` | `RemoveLobbyGamePacket::~RemoveLobbyGamePacket` | 🟢 | dtor | Destructor body: writes BasePacket::vftable into the object vtable slot and returns; the object storage itself is released by the caller (00923210 in the lobby dispatch path) or an EH unwind funclet. |
| `008b7b20` | `RemoveLobbyGamePacket::ScalarDeletingDtor` | ✅ | dtor | Restores BasePacket::vftable and frees the object when the deleting flag bit is set. |
