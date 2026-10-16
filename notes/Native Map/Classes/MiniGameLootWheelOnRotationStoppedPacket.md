# Class: MiniGameLootWheelOnRotationStoppedPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009aa0c0` | `MiniGameLootWheelOnRotationStoppedPacket::MiniGameLootWheelOnRotationStoppedPac…` | ✅ | ctor | Constructor: calls the minigame base constructor FUN_008a4420 with sub-opcode 0x2e and three -1 header words, then installs MiniGameLootWheelOnRotationStoppedPacket::vftable. |
| `009ac870` | `DeletingDtor_009ac870` | 🟢 | dtor | Scalar deleting destructor for a BasePacket-derived minigame packet: rewrites *this to BasePacket::vftable and, when bit 0 of param_2 is set, frees the object through the _free thunk; returns the object pointer otherwis… |
