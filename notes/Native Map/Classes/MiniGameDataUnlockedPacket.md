# Class: MiniGameDataUnlockedPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009a98c0` | `MiniGameDataUnlockedPacket::MiniGameDataUnlockedPacket` | ✅ | ctor | Constructor: calls the minigame base constructor FUN_008a4420 with sub-opcode 2 and three -1 payload words, then installs MiniGameDataUnlockedPacket::vftable and zeroes this[6]/this[7]. |
| `009ac540` | `DeletingDtor_009ac540` | 🟢 | dtor | Scalar deleting destructor for a BasePacket-derived minigame packet: rewrites *this to BasePacket::vftable and, when bit 0 of param_2 is set, frees the object through the _free thunk; returns the object pointer otherwis… |
