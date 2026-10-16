# Class: MiniGameMessagePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009a9bc0` | `MiniGameMessagePacket::MiniGameMessagePacket` | ✅ | ctor | Constructor: calls the minigame base constructor FUN_008a4420 with sub-opcode 0xf and two -1 payload words, installs MiniGameMessagePacket::vftable, stores param_3 at this[6] and param_4 as a byte at this+0x1c. |
| `009ac690` | `DeletingDtor_009ac690` | 🟢 | dtor | Scalar deleting destructor for a BasePacket-derived minigame packet: rewrites *this to BasePacket::vftable and, when bit 0 of param_2 is set, frees the object through the _free thunk; returns the object pointer otherwis… |
