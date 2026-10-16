# Class: PacketDismountRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009aa410` | `PacketDismountRequest::PacketDismountRequest` | ✅ | ctor | Constructor: calls the MountBasePacket base constructor FUN_0089f9c0(3) (which writes the mount family opcode and sub-opcode 3) and installs PacketDismountRequest::vftable. |
| `009ac9d0` | `DeletingDtor_009ac9d0` | 🟢 | dtor | Scalar deleting destructor for a BasePacket-derived mount packet: rewrites *this to BasePacket::vftable and, when bit 0 of param_2 is set, frees the object through the _free thunk; returns the object pointer otherwise. … |
