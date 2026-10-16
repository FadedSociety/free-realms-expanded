# Class: ObjectiveFailPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009aa680` | `ObjectiveFailPacket::ObjectiveFailPacket` | ✅ | ctor | Constructor: calls the family constructor FUN_009aa4c0(4) (opcode 45, sub-opcode 4), installs ObjectiveFailPacket::vftable and stores param_2 at this[3]. |
| `009acb00` | `DeletingDtor_009acb00` | 🟢 | dtor | Scalar deleting destructor for a BasePacket-derived objective packet: rewrites *this to BasePacket::vftable and, when bit 0 of param_2 is set, frees the object through the _free thunk; returns the object pointer otherwi… |
