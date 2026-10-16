# Class: UpdateLobbyGamePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a3fe0` | `UpdateLobbyGamePacket::UpdateLobbyGamePacket` | ✅ | ctor | Constructor for lobby sub-opcode 4: calls the opcode-101 family constructor FUN_008a3ce0(4), installs UpdateLobbyGamePacket::vftable; body is wrapped in an MSVC SEH frame that restores ExceptionList. |
| `008a4040` | `UpdateLobbyGamePacket::~UpdateLobbyGamePacket` | 🟢 | dtor | Destructor body: writes BasePacket::vftable into the object vtable slot and returns; the object storage itself is released by the caller (00923210 in the lobby dispatch path) or an EH unwind funclet. |
| `008b7a00` | `UpdateLobbyGamePacket::ScalarDeletingDtor` | ✅ | dtor | Restores BasePacket::vftable and frees the object when the deleting flag bit is set. |
