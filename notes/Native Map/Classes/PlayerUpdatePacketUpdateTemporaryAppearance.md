# Class: PlayerUpdatePacketUpdateTemporaryAppearance

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a0aa0` | `PlayerUpdatePacketUpdateTemporaryAppearance::~PlayerUpdatePacketUpdateTemporary…` | 🟢 | dtor | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; called from dispatcher case 0xe, from 0096c300 and from three EH funclets. |
| `008b4370` | `PlayerUpdatePacketUpdateTemporaryAppearance::PlayerUpdatePacketUpdateTemporaryA…` | ✅ | ctor | Calls the opcode-35 family constructor 0089fbb0(0xe) and installs PlayerUpdatePacketUpdateTemporaryAppearance::vftable; no field is initialised. |
| `008b43d0` | `PlayerUpdatePacketUpdateTemporaryAppearance::ScalarDeletingDestructor` | 🟢 | dtor | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
