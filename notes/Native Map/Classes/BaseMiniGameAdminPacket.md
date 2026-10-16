# Class: BaseMiniGameAdminPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009a97c0` | `BaseMiniGameAdminPacket::BaseMiniGameAdminPacket` | ✅ | ctor | Constructor: installs BasePacket::vftable, writes this[1] = 0x5f (base opcode 95), copies param_2 into this[2], then installs BaseMiniGameAdminPacket::vftable and stores param_3/param_4 into this[3]/this[4]. |
| `009ac450` | `DeletingDtor_009ac450` | 🟢 | dtor | Scalar deleting destructor for a BasePacket-derived minigame packet: rewrites *this to BasePacket::vftable and, when bit 0 of param_2 is set, frees the object through the _free thunk; returns the object pointer otherwis… |
