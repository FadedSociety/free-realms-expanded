# Class: MiniGameGroupInfoPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009b8580` | `MiniGameGroupInfoPacket::MiniGameGroupInfoPacket` | ✅ | ctor | Mini-game packet constructor: calls FUN_008a4420(0x32,0,-1,-1) so the packet is family 39 with sub-opcode 50 (0x32) and the three id words 0/-1/-1, installs MiniGameGroupInfoPacket::vftable and then initialises the embe… |
| `009b85f0` | `MiniGameGroupInfoPacket::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor for the family-39 group-info packet: calls the class destructor body 009b6140, which resets BasePacket::vftable and, through 009b6070, destroys the SoeUtil::Array<MiniGameGroupElementInfo> at … |
