# Class: PlayerUpdatePacketUpdateCharacterState

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a0b90` | `PlayerUpdatePacketUpdateCharacterState::~PlayerUpdatePacketUpdateCharacterState` | ✅ | dtor | Destructor body: restores PlayerUpdatePacketUpdateCharacterState::vftable, then BasePlayerUpdatePacket::vftable, then BasePacket::vftable. |
| `008a0bf0` | `PlayerUpdatePacketUpdateCharacterState::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor: restores the character-state, BasePlayerUpdatePacket and BasePacket vtables inside an SEH frame and calls _free(this) when param_2 bit 0 is set. |
| `008b4510` | `PlayerUpdatePacketUpdateCharacterState::PlayerUpdatePacketUpdateCharacterState` | ✅ | ctor | Calls the opcode-35 family constructor 0089fbb0(0x14) and installs PlayerUpdatePacketUpdateCharacterState::vftable; no field is initialised. |
