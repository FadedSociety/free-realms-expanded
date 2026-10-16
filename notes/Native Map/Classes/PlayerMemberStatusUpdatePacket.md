# Class: PlayerMemberStatusUpdatePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a1950` | `PlayerMemberStatusUpdatePacket::~PlayerMemberStatusUpdatePacket` | 🟢 | dtor | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; it is the last call of dispatcher case 0x42 and also runs from EH funclet 01591838. |
| `008b5ba0` | `PlayerMemberStatusUpdatePacket::PlayerMemberStatusUpdatePacket ⚠downgrade` | 🟢 | ctor | Calls the opcode-35 family constructor 0089fbb0(0x42), installs PlayerMemberStatusUpdatePacket::vftable, stores the global pair DAT_01be4850/DAT_01be4854 into +0x10/+0x14 and zeroes the byte at +0x18. |
| `008b5c10` | `PlayerMemberStatusUpdatePacket::ScalarDeletingDestructor` | 🟢 | dtor | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
