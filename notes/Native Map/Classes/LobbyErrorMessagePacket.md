# Class: LobbyErrorMessagePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a42a0` | `LobbyErrorMessagePacket::LobbyErrorMessagePacket` | ✅ | ctor | Constructor for lobby sub-opcode 11 (0xb): calls the opcode-101 family constructor FUN_008a3ce0(0xb), installs LobbyErrorMessagePacket::vftable and stores the single payload word at +0xc; body is wrapped in an MSVC SEH … |
| `008a4310` | `LobbyErrorMessagePacket::~LobbyErrorMessagePacket` | 🟢 | dtor | Destructor body: writes BasePacket::vftable into the object vtable slot and returns; the caller releases the storage. |
| `008b7b80` | `LobbyErrorMessagePacket::ScalarDeletingDtor` | ✅ | dtor | Restores BasePacket::vftable and frees the object when the deleting flag bit is set. |
