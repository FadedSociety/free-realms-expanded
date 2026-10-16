# Class: PlayerTitleSNSUpdatePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bb6ba0` | `PlayerTitleSNSUpdatePacket::PlayerTitleSNSUpdatePacket` | ✅ | ctor | Constructor: runs the BasePlayerTitlePacket family constructor 00b491a0 with sub-opcode 7 (which stores base opcode 0x98 = 152), installs PlayerTitleSNSUpdatePacket::vftable and initialises the embedded SoeUtil::List<in… |
| `00bb6c10` | `PlayerTitleSNSUpdatePacket::~PlayerTitleSNSUpdatePacket` | ✅ | dtor | Destructor: installs PlayerTitleSNSUpdatePacket::vftable, rewrites the SoeUtil::List<int,-1> vtable at +0x18 and clears that list with 007798c0, then installs BasePlayerTitlePacket::vftable and BasePacket::vftable at +0. |
| `00bb6c80` | `PlayerTitleSNSUpdatePacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor at vtable slot 0: runs the destructor body 00bb6c10 and, when bit 0 of the flag argument is set, releases this through the CRT free thunk; returns this. |
