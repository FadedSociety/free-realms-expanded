# Class: PlayerUpdatePacketHitPointAndShieldsModification

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a10b0` | `PlayerUpdatePacketHitPointAndShieldsModification::Dtor` | 🟢 | dtor | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; it is the last call of dispatcher case 0x3a and also runs from EH funclet 015914dd. |
| `008b4e10` | `PlayerUpdatePacketHitPointAndShieldsModification::Ctor` | ✅ | ctor | Calls the opcode-35 family constructor 0089fbb0(0x3a) and installs PlayerUpdatePacketHitPointAndShieldsModification::vftable; no field is initialised. |
| `008b4e70` | `PlayerUpdatePacketHitPointAndShieldsModification::ScalarDeletingDestructor` | ✅ | dtor | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
