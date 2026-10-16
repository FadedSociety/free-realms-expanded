# Class: PlayerUpdatePacketKnockback

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a08b0` | `PlayerUpdatePacketKnockback::~PlayerUpdatePacketKnockback` | 🟢 | dtor | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; called from dispatcher case 4 and EH funclet 01591590. |
| `008b4190` | `PlayerUpdatePacketKnockback::PlayerUpdatePacketKnockback ⚠downgrade` | 🟢 | ctor | Calls the opcode-35 family constructor 0089fbb0(4), installs PlayerUpdatePacketKnockback::vftable, stores the global pair DAT_01be4850/DAT_01be4854 into +0x10/+0x14, zeroes +0x18 and writes the global float DAT_0175b420… |
| `008b4210` | `PlayerUpdatePacketKnockback::ScalarDeletingDestructor` | 🟢 | dtor | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
