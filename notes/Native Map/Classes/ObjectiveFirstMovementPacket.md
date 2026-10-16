# Class: ObjectiveFirstMovementPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009aabd0` | `ObjectiveFirstMovementPacket::ObjectiveFirstMovementPacket` | ✅ | ctor | Constructor: calls the family constructor FUN_009aa4c0(0xb) (opcode 45, sub-opcode 11), installs ObjectiveFirstMovementPacket::vftable and sets this[3] = 0xffffffff. |
| `009acdf0` | `DeletingDtor_009acdf0` | 🟢 | dtor | Scalar deleting destructor for a BasePacket-derived objective packet: rewrites *this to BasePacket::vftable and, when bit 0 of param_2 is set, frees the object through the _free thunk; returns the object pointer otherwi… |
