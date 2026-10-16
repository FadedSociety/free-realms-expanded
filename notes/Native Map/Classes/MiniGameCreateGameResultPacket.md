# Class: MiniGameCreateGameResultPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009aa230` | `MiniGameCreateGameResultPacket::MiniGameCreateGameResultPacket` | ✅ | ctor | Constructor: calls the minigame base constructor FUN_008a4420 with sub-opcode 0x43 and three -1 header words, installs MiniGameCreateGameResultPacket::vftable and clears the byte at this+0x18. |
| `009ac930` | `DeletingDtor_009ac930 → MiniGameCreateGameResultPacket::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor for a BasePacket-derived minigame packet: rewrites *this to BasePacket::vftable and, when bit 0 of param_2 is set, frees the object through the _free thunk; returns the object pointer otherwis… |
