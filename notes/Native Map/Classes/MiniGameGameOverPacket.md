# Class: MiniGameGameOverPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009a9ab0` | `MiniGameGameOverPacket::MiniGameGameOverPacket` | ✅ | ctor | Constructor: calls the minigame base constructor FUN_008a4420 with sub-opcode 0x12, writes byte param_3 at this+0x18, installs MiniGameGameOverPacket::vftable and stores param_4/param_5/param_6 into this[7]/this[8]/this… |
| `009ac630` | `DeletingDtor_009ac630` | 🟢 | dtor | Scalar deleting destructor for a BasePacket-derived minigame packet: rewrites *this to BasePacket::vftable and, when bit 0 of param_2 is set, frees the object through the _free thunk; returns the object pointer otherwis… |
