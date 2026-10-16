# Class: SendLobbyToClientPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a4090` | `SendLobbyToClientPacket::SendLobbyToClientPacket` | ✅ | ctor | Constructor for lobby sub-opcode 6: calls the opcode-101 family constructor FUN_008a3ce0(6), installs SendLobbyToClientPacket::vftable; body is wrapped in an MSVC SEH frame that restores ExceptionList. |
| `008a40f0` | `SendLobbyToClientPacket::~SendLobbyToClientPacket` | 🟢 | dtor | Destructor body: writes BasePacket::vftable into the object vtable slot and returns; the object storage itself is released by the caller (00923210 in the lobby dispatch path) or an EH unwind funclet. |
| `008b7a60` | `SendLobbyToClientPacket::ScalarDeletingDtor` | ✅ | dtor | Restores BasePacket::vftable and frees the object when the deleting flag bit is set. |
