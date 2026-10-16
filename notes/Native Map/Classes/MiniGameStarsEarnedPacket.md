# Class: MiniGameStarsEarnedPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009a9cb0` | `MiniGameStarsEarnedPacket::MiniGameStarsEarnedPacket` | ✅ | ctor | Constructor: calls the minigame base constructor FUN_008a4420 with sub-opcode 0x15 and param_2/param_3 payload words, installs MiniGameStarsEarnedPacket::vftable and stores param_4 as a byte at this+0x18. |
| `009ac6f0` | `DeletingDtor_009ac6f0` | 🟢 | dtor | Scalar deleting destructor for a BasePacket-derived minigame packet: rewrites *this to BasePacket::vftable and, when bit 0 of param_2 is set, frees the object through the _free thunk; returns the object pointer otherwis… |
