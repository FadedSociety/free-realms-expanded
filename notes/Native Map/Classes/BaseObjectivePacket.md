# Class: BaseObjectivePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009aa4c0` | `BaseObjectivePacket::BaseObjectivePacket` | ✅ | ctor | Constructor: installs BasePacket::vftable, writes this[1] = 0x2d (family/opcode 45), stores the sub-opcode param_2 at this[2] and installs BaseObjectivePacket::vftable. |
| `009aca40` | `DeletingDtor_009aca40` | 🟢 | dtor | Scalar deleting destructor for a BasePacket-derived objective packet: rewrites *this to BasePacket::vftable and, when bit 0 of param_2 is set, frees the object through the _free thunk; returns the object pointer otherwi… |
