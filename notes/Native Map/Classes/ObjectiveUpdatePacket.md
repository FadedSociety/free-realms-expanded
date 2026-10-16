# Class: ObjectiveUpdatePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009aa830` | `ObjectiveUpdatePacket::ObjectiveUpdatePacket` | ✅ | ctor | Constructor: calls the family constructor FUN_009aa4c0(2) (opcode 45, sub-opcode 2), installs ObjectiveUpdatePacket::vftable and stores param_2/param_3 into this[3]/this[4]. |
| `009acbc0` | `DeletingDtor_009acbc0` | 🟢 | dtor | Scalar deleting destructor for a BasePacket-derived objective packet: rewrites *this to BasePacket::vftable and, when bit 0 of param_2 is set, frees the object through the _free thunk; returns the object pointer otherwi… |
