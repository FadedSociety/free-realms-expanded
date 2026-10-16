# Class: TradingCardStartGamePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009a9de0` | `TradingCardStartGamePacket::TradingCardStartGamePacket` | ✅ | ctor | Constructor: calls the minigame base constructor FUN_008a4420 with sub-opcode 0x28, placing param_2 and param_5 in the family header, installs TradingCardStartGamePacket::vftable and stores param_3/param_4 into this[6]/… |
| `009ac750` | `DeletingDtor_009ac750` | 🟢 | dtor | Scalar deleting destructor for a BasePacket-derived minigame packet: rewrites *this to BasePacket::vftable and, when bit 0 of param_2 is set, frees the object through the _free thunk; returns the object pointer otherwis… |
