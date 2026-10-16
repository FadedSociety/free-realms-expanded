# Class: ShowLobbyUiPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a4360` | `ShowLobbyUiPacket::ShowLobbyUiPacket` | ✅ | ctor | Constructor for lobby sub-opcode 12 (0xc): calls the opcode-101 family constructor FUN_008a3ce0(0xc), installs ShowLobbyUiPacket::vftable and stores the two payload words at +0xc and +0x10; body is wrapped in an MSVC SE… |
| `008a43d0` | `ShowLobbyUiPacket::~ShowLobbyUiPacket` | 🟢 | dtor | Destructor body: writes BasePacket::vftable into the object vtable slot and returns; the caller releases the storage. |
| `008b7be0` | `ShowLobbyUiPacket::ScalarDeletingDtor` | ✅ | dtor | Restores BasePacket::vftable and frees the object when the deleting flag bit is set. |
