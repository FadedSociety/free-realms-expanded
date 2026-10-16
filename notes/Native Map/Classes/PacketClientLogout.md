# Class: PacketClientLogout

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089b120` | `PacketClientLogoutCtor` | ✅ | ctor | Constructor of PacketClientLogout: installs BasePacket::vftable, stores opcode 0x10 (16) at +4, then installs PacketClientLogout::vftable. |
| `0089b180` | `PacketClientLogoutDtor` | ✅ | dtor | Destructor body: installs PacketClientLogout::vftable then BasePacket::vftable; no member cleanup. |
| `0089b1d0` | `PacketClientLogoutScalarDtor` | ✅ | dtor | Scalar deleting destructor: installs the derived then the base vtable and calls _free when bit 0 of param_2 is set. |
