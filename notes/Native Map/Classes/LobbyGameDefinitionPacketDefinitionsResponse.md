# Class: LobbyGameDefinitionPacketDefinitionsResponse

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a4b00` | `LobbyGameDefinitionPacketDefinitionsResponse::LobbyGameDefinitionPacketDefiniti…` | ✅ | ctor | Constructor taking two payload words: calls FUN_008a49d0(2) so the packet is opcode 102 with sub-opcode 2, installs LobbyGameDefinitionPacketDefinitionsResponse::vftable and stores the arguments at +0xc and +0x10; SEH f… |
| `008a4b70` | `LobbyGameDefinitionPacketDefinitionsResponse::~LobbyGameDefinitionPacketDefinit…` | ✅ | dtor | Destructor body: restores LobbyGameDefinitionPacketDefinitionsResponse::vftable, then BaseLobbyGameDefinitionPacket::vftable, then BasePacket::vftable; caller 0091e060 and EH funclet 0158e070 call it. |
| `008a4bd0` | `LobbyGameDefinitionPacketDefinitionsResponse::ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor: in an SEH frame restores the response, BaseLobbyGameDefinitionPacket and BasePacket vtables, then calls _free(this) when param_2 bit 0 is set. |
