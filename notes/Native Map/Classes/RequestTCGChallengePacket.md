# Class: RequestTCGChallengePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009a9ec0` | `RequestTCGChallengePacket::RequestTCGChallengePacket` | ✅ | ctor | Constructor: calls the minigame base constructor FUN_008a4420 with sub-opcode 0x26 and param_2 in the third header word, then installs RequestTCGChallengePacket::vftable. |
| `009ac7b0` | `DeletingDtor_009ac7b0` | 🟢 | dtor | Scalar deleting destructor for a BasePacket-derived minigame packet: rewrites *this to BasePacket::vftable and, when bit 0 of param_2 is set, frees the object through the _free thunk; returns the object pointer otherwis… |
