# Class: MiniGameKnockOutPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009a99e0` | `MiniGameKnockOutPacket::MiniGameKnockOutPacket` | ✅ | ctor | Constructor: calls the minigame base constructor FUN_008a4420 with sub-opcode 0x17, three payload words (param_2 plus two -1 words), then installs MiniGameKnockOutPacket::vftable and stores param_3/param_4 into this[6]/… |
| `009ac5d0` | `DeletingDtor_009ac5d0` | 🟢 | dtor | Scalar deleting destructor for a BasePacket-derived minigame packet: rewrites *this to BasePacket::vftable and, when bit 0 of param_2 is set, frees the object through the _free thunk; returns the object pointer otherwis… |
