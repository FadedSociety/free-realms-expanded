# Class: PlayerUpdatePacketMoveOnRelativeRail

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a1640` | `PlayerUpdatePacketMoveOnRelativeRail::~PlayerUpdatePacketMoveOnRelativeRail` | 🟢 | dtor | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; it is the last call of dispatcher case 0x37 and also runs from EH funclet 015917b4. |
| `008b55f0` | `PlayerUpdatePacketMoveOnRelativeRail::PlayerUpdatePacketMoveOnRelativeRail ⚠dow…` | 🟢 | ctor | Calls the opcode-35 family constructor 0089fbb0(0x37), installs PlayerUpdatePacketMoveOnRelativeRail::vftable, stores the global pair DAT_01be4850/DAT_01be4854 into +0x10/+0x14 and zeroes +0x18, +0x1c, +0x20, +0x24 and … |
| `008b5670` | `PlayerUpdatePacketMoveOnRelativeRail::ScalarDeletingDestructor` | ✅ | dtor | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
