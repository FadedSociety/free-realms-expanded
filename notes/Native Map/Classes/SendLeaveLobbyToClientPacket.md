# Class: SendLeaveLobbyToClientPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a4140` | `SendLeaveLobbyToClientPacket::SendLeaveLobbyToClientPacket` | ✅ | ctor | Constructor for lobby sub-opcode 7: calls the opcode-101 family constructor FUN_008a3ce0(7), installs SendLeaveLobbyToClientPacket::vftable; body is wrapped in an MSVC SEH frame that restores ExceptionList. |
| `008a41a0` | `SendLeaveLobbyToClientPacket::~SendLeaveLobbyToClientPacket` | 🟢 | dtor | Destructor body: writes BasePacket::vftable into the object vtable slot and returns; the object storage itself is released by the caller (00923210 in the lobby dispatch path) or an EH unwind funclet. |
| `008b7ac0` | `SendLeaveLobbyToClientPacket::ScalarDeletingDtor` | ✅ | dtor | Restores BasePacket::vftable and frees the object when the deleting flag bit is set. |
