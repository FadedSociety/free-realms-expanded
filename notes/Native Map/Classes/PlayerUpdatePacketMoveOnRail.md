# Class: PlayerUpdatePacketMoveOnRail

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a1570` | `PlayerUpdatePacketMoveOnRail::~PlayerUpdatePacketMoveOnRail` | 🟢 | dtor | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; it is the last call of dispatcher case 0x35 and also runs from EH funclet 0159179e. |
| `008b53e0` | `PlayerUpdatePacketMoveOnRail::PlayerUpdatePacketMoveOnRail ⚠downgrade` | 🟢 | ctor | Calls the opcode-35 family constructor 0089fbb0(0x35), installs PlayerUpdatePacketMoveOnRail::vftable, stores the global pair DAT_01be4850/DAT_01be4854 into +0x10/+0x14 and zeroes +0x18 and +0x1c. |
| `008b5460` | `PlayerUpdatePacketMoveOnRail::ScalarDeletingDestructor` | 🟢 | dtor | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
