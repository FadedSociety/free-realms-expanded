# Class: PlayerUpdatePacketSetCollidable

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a18a0` | `PlayerUpdatePacketSetCollidable::~PlayerUpdatePacketSetCollidable` | 🟢 | dtor | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; it is the last call of dispatcher case 0x32 and also runs from EH funclet 0159177d. |
| `008b59c0` | `PlayerUpdatePacketSetCollidable::PlayerUpdatePacketSetCollidable` | ✅ | ctor | Calls the opcode-35 family constructor 0089fbb0(0x32) and installs PlayerUpdatePacketSetCollidable::vftable; no field is initialised. |
| `008b5a20` | `PlayerUpdatePacketSetCollidable::ScalarDeletingDestructor` | 🟢 | dtor | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
