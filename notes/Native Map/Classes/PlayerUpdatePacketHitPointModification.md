# Class: PlayerUpdatePacketHitPointModification

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a1060` | `PlayerUpdatePacketHitPointModification::~PlayerUpdatePacketHitPointModification` | 🟢 | dtor | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; it is the last call of dispatcher case 0x23 and also runs from EH funclet 015914d2. |
| `008b4d40` | `PlayerUpdatePacketHitPointModification::PlayerUpdatePacketHitPointModification …` | 🟢 | ctor | Calls the opcode-35 family constructor 0089fbb0(0x23) and installs PlayerUpdatePacketHitPointModification::vftable; no field is initialised. |
| `008b4da0` | `PlayerUpdatePacketHitPointModification::ScalarDeletingDestructor` | 🟢 | dtor | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
