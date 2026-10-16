# Class: ObjectiveCompletePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009aa590` | `ObjectiveCompletePacket::ObjectiveCompletePacket` | ✅ | ctor | Constructor: calls the family constructor FUN_009aa4c0(3) (opcode 45, sub-opcode 3), installs ObjectiveCompletePacket::vftable and stores param_2/param_3/param_4 into this[3]/this[4]/this[5]. |
| `009acaa0` | `DeletingDtor_009acaa0` | 🟢 | dtor | Scalar deleting destructor for a BasePacket-derived objective packet: rewrites *this to BasePacket::vftable and, when bit 0 of param_2 is set, frees the object through the _free thunk; returns the object pointer otherwi… |
