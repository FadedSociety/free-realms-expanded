# Class: ObjectiveLookAtPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009acc20` | `ObjectiveLookAtPacket::ObjectiveLookAtPacket ⚠downgrade` | 🟢 | ctor | Constructor: calls the family constructor FUN_009aa4c0(6) (opcode 45, sub-opcode 6), installs ObjectiveLookAtPacket::vftable, sets this[3] = -1, copies the two globals DAT_01be59a8/DAT_01be59ac into this[4]/this[5], zer… |
| `009acca0` | `DeletingDtor_009acca0` | 🟢 | dtor | Scalar deleting destructor for a BasePacket-derived objective packet: rewrites *this to BasePacket::vftable and, when bit 0 of param_2 is set, frees the object through the _free thunk; returns the object pointer otherwi… |
