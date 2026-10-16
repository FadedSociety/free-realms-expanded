# Class: PacketSendSelfToClient

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089b240` | `PacketSendSelfToClientCtor` | ✅ | ctor | Constructor of PacketSendSelfToClient: installs BasePacket::vftable, stores opcode 0xc (12) at +4, installs the derived vtable and stores the two payload words param_2 and param_3 at +8 and +0xc. |
| `0089b2b0` | `PacketSendSelfToClientDtor` | ✅ | dtor | Destructor body: installs PacketSendSelfToClient::vftable then BasePacket::vftable; no member cleanup. |
| `0089b300` | `PacketSendSelfToClientScalarDtor` | ✅ | dtor | Scalar deleting destructor: installs the derived then the base vtable and calls _free when bit 0 of param_2 is set. |
