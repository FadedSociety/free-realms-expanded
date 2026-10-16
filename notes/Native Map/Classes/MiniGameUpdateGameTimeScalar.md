# Class: MiniGameUpdateGameTimeScalar

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009aa170` | `MiniGameUpdateGameTimeScalar::MiniGameUpdateGameTimeScalar` | ✅ | ctor | Constructor: calls the minigame base constructor FUN_008a4420 with sub-opcode 0x14 and three -1 header words, installs MiniGameUpdateGameTimeScalar::vftable and copies the global _DAT_0175b420 into this[6]. |
| `009ac8d0` | `DeletingDtor_009ac8d0 → MiniGameUpdateGameTimeScalar::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor for a BasePacket-derived minigame packet: rewrites *this to BasePacket::vftable and, when bit 0 of param_2 is set, frees the object through the _free thunk; returns the object pointer otherwis… |
