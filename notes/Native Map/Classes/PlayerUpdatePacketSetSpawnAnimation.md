# Class: PlayerUpdatePacketSetSpawnAnimation

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a14d0` | `PlayerUpdatePacketSetSpawnAnimation::~PlayerUpdatePacketSetSpawnAnimation` | 🟢 | dtor | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; it is the last call of dispatcher case 0x2d and also runs from EH funclet 0159156f. |
| `008b5310` | `PlayerUpdatePacketSetSpawnAnimation::PlayerUpdatePacketSetSpawnAnimation` | ✅ | ctor | Calls the opcode-35 family constructor 0089fbb0(0x2d) and installs PlayerUpdatePacketSetSpawnAnimation::vftable; no field is initialised. |
| `008b5370` | `PlayerUpdatePacketSetSpawnAnimation::ScalarDeletingDestructor` | 🟢 | dtor | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
