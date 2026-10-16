# Class: LeaveLobbyGamePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a3e60` | `LeaveLobbyGamePacket::LeaveLobbyGamePacket` | ✅ | ctor | Constructor for lobby sub-opcode 2: calls the opcode-101 family constructor FUN_008a3ce0(2), installs LeaveLobbyGamePacket::vftable and stores the 2 payload word(s) at +0xc/+0x10; body is wrapped in an MSVC SEH frame th… |
| `008a3ed0` | `LeaveLobbyGamePacket::~LeaveLobbyGamePacket` | 🟢 | dtor | Destructor body: writes BasePacket::vftable into the object vtable slot and returns; the object storage itself is released by the caller (00923210 in the lobby dispatch path) or an EH unwind funclet. |
| `008b7940` | `LeaveLobbyGamePacket::ScalarDeletingDtor` | ✅ | dtor | Restores BasePacket::vftable and frees the object when the deleting flag bit is set. |
