# Class: ObjectiveClientClearPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009aaa20` | `ObjectiveClientClearPacket::ObjectiveClientClearPacket` | ✅ | ctor | Constructor: calls the family constructor FUN_009aa4c0(8) (opcode 45, sub-opcode 8), installs ObjectiveClientClearPacket::vftable and stores param_2 at this[3]. |
| `009acd20` | `DeletingDtor_009acd20` | 🟢 | dtor | Scalar deleting destructor for a BasePacket-derived objective packet: rewrites *this to BasePacket::vftable and, when bit 0 of param_2 is set, frees the object through the _free thunk; returns the object pointer otherwi… |
