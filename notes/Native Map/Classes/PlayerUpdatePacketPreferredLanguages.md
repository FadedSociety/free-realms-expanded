# Class: PlayerUpdatePacketPreferredLanguages

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a11c0` | `PlayerUpdatePacketPreferredLanguages::~PlayerUpdatePacketPreferredLanguages` | 🟢 | dtor | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; it is the last call of dispatcher case 0x26 and also runs from EH funclet 01591725. |
| `008b4f50` | `PlayerUpdatePacketPreferredLanguages::PlayerUpdatePacketPreferredLanguages ⚠dow…` | 🟢 | ctor | Calls the opcode-35 family constructor 0089fbb0(0x26), installs PlayerUpdatePacketPreferredLanguages::vftable, stores the global pair DAT_01be4850/DAT_01be4854 into +0x10/+0x14 and zeroes +0x18. |
| `008b4fc0` | `PlayerUpdatePacketPreferredLanguages::ScalarDeletingDestructor` | 🟢 | dtor | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
