# Class: LobbyGameDefinitionPacketDefinitionsRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a221d0` | `LobbyGameDefinitionPacketDefinitionsRequest::LobbyGameDefinitionPacketDefinitio…` | ✅ | ctor | Constructs the packet on the stack or heap object passed in ecx: calls the lobby-game-definition family constructor 008a49d0 with sub-opcode 1 (which stores opcode 0x66 and the sub-opcode) and then stores LobbyGameDefin… |
| `00a22230` | `LobbyGameDefinitionPacketDefinitionsRequest::Dtor` | 🟢 | dtor | Destructor body: rewrites the object's vtable pointer back through LobbyGameDefinitionPacketDefinitionsRequest::vftable, BaseLobbyGameDefinitionPacket::vftable and BasePacket::vftable in that order; no members are relea… |
| `00a22290` | `LobbyGameDefinitionPacketDefinitionsRequest::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: restores the LobbyGameDefinitionPacketDefinitionsRequest, BaseLobbyGameDefinitionPacket and BasePacket vtables and then calls _free on the object when bit 0 of the flag argument is set; other… |
