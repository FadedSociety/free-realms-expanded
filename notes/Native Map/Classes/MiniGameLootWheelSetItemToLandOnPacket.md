# Class: MiniGameLootWheelSetItemToLandOnPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009b9e20` | `MiniGameLootWheelSetItemToLandOnPacket::MiniGameLootWheelSetItemToLandOnPacket` | ✅ | ctor | Constructs the family-39 sub-opcode 45 (0x2d) loot-wheel packet: calls FUN_008a4420(0x2d,-1,-1,-1), installs MiniGameLootWheelSetItemToLandOnPacket::vftable and initialises one embedded member through FUN_0091c110. |
| `009b9e90` | `MiniGameLootWheelSetItemToLandOnPacket::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor for the 39/45 loot-wheel packet: calls the packet destructor body 009aa050, the same destructor the dispatcher arm for case 0x2d uses to close its in-place packet. Then frees the object throug… |
