# Class: BaseLobbyGameDefinitionPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a4970` | `BaseLobbyGameDefinitionPacket::BaseLobbyGameDefinitionPacket` | ✅ | ctor | Lobby-game-definition family constructor with a zero sub-opcode: installs BasePacket::vftable, stores opcode 102 (0x66) at +4, installs BaseLobbyGameDefinitionPacket::vftable and zeroes +8; no calls. |
| `008a49d0` | `BaseLobbyGameDefinitionPacket::BaseLobbyGameDefinitionPacketSub` | ✅ | ctor | Lobby-game-definition family constructor taking the sub-opcode: installs BasePacket::vftable, stores opcode 102 (0x66) at +4, installs BaseLobbyGameDefinitionPacket::vftable and stores the sub-opcode argument at +8; no … |
| `008a4a40` | `BaseLobbyGameDefinitionPacket::~BaseLobbyGameDefinitionPacket` | ✅ | dtor | Destructor body: writes BaseLobbyGameDefinitionPacket::vftable then BasePacket::vftable into the object vtable slot and returns; eight EH funclets (0157def0 and neighbours) call it. |
| `008a4a90` | `BaseLobbyGameDefinitionPacket::ScalarDeletingDestructor ⚠downgrade` | 🟢 | dtor | Scalar deleting destructor: in an SEH frame restores BaseLobbyGameDefinitionPacket::vftable then BasePacket::vftable and calls _free(this) when param_2 bit 0 is set. |
