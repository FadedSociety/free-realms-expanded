# Class: JoinLobbyGamePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a3da0` | `JoinLobbyGamePacket::JoinLobbyGamePacket` | ✅ | ctor | Constructor for lobby sub-opcode 1: calls the opcode-101 family constructor FUN_008a3ce0(1), installs JoinLobbyGamePacket::vftable and stores the 2 payload word(s) at +0xc/+0x10; body is wrapped in an MSVC SEH frame tha… |
| `008a3e10` | `JoinLobbyGamePacket::~JoinLobbyGamePacket` | 🟢 | dtor | Destructor body: writes BasePacket::vftable into the object vtable slot and returns; the object storage itself is released by the caller (00923210 in the lobby dispatch path) or an EH unwind funclet. |
| `008b78e0` | `JoinLobbyGamePacket::ScalarDeletingDtor` | ✅ | dtor | Restores BasePacket::vftable and frees the object when the deleting flag bit is set. |
