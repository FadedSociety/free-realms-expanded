# Class: ObjectiveActivatePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009aa750` | `ObjectiveActivatePacket::ObjectiveActivatePacket` | ✅ | ctor | Constructor: calls the family constructor FUN_009aa4c0(1) (opcode 45, sub-opcode 1), installs ObjectiveActivatePacket::vftable and stores param_2/param_3 into this[3]/this[4]. |
| `009acb60` | `DeletingDtor_009acb60` | 🟢 | dtor | Scalar deleting destructor for a BasePacket-derived objective packet: rewrites *this to BasePacket::vftable and, when bit 0 of param_2 is set, frees the object through the _free thunk; returns the object pointer otherwi… |
