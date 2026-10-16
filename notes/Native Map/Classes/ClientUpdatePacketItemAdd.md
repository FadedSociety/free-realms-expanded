# Class: ClientUpdatePacketItemAdd

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089d410` | `ClientUpdatePacketItemAddCtor` | ✅ | ctor | Constructor of ClientUpdatePacketItemAdd: runs the base constructor FUN_0089d180 with sub-opcode 2, then installs ClientUpdatePacketItemAdd::vftable. |
| `0089d470` | `ClientUpdatePacketItemAddDtor` | ✅ | dtor | Destructor body: installs ClientUpdatePacketItemAdd::vftable, then BaseClientUpdatePacket::vftable and BasePacket::vftable; no member cleanup. |
| `0089d4d0` | `ClientUpdatePacketItemAddScalarDtor` | ✅ | dtor | Scalar deleting destructor: installs the three vtables (derived, family, base) and calls _free when bit 0 of param_2 is set. |
