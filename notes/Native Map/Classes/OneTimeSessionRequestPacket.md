# Class: OneTimeSessionRequestPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089b510` | `OneTimeSessionRequestPacketCtor` | 🟢 | ctor | Constructor of OneTimeSessionRequestPacket: installs BasePacket::vftable, stores opcode 0x75 (117) at +4, installs the derived vtable and clears the payload word at +8. |
| `0089b570` | `OneTimeSessionRequestPacketDtor` | ✅ | dtor | Destructor body: installs OneTimeSessionRequestPacket::vftable then BasePacket::vftable; no member cleanup. |
| `0089b5c0` | `OneTimeSessionRequestPacketScalarDtor` | ✅ | dtor | Scalar deleting destructor: installs the derived then the base vtable and calls _free when bit 0 of param_2 is set. |
