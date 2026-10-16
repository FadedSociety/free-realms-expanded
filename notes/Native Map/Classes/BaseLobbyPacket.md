# Class: BaseLobbyPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a3ce0` | `BaseLobbyPacket::BaseLobbyPacket` | ✅ | ctor | Lobby packet-family constructor: installs BasePacket::vftable at +0, stores the base opcode 101 (0x65) at +4, installs BaseLobbyPacket::vftable, then stores the sub-opcode argument at +8; no calls and no other writes. |
| `008a3d50` | `BaseLobbyPacket::~BaseLobbyPacket` | 🟢 | dtor | Destructor body restores BasePacket::vftable and returns. Dispatcher 00923210 invokes it for its packet temporary; that caller does not free the temporary here, and 39 EH unwind funclets also call this cleanup body. |
| `008b7880` | `BaseLobbyPacket::ScalarDeletingDtor` | ✅ | dtor | Restores BasePacket::vftable and frees the object when the deleting flag bit is set. |
