# Class: FactoryPacketListToolsResponse

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009e41b0` | `FactoryPacketListToolsResponse::Dtor` | ✅ | dtor | Calls 009e4140 with ECX=this+0x0c to destroy the embedded tool map, then restores BaseFactoryPacket and BasePacket vtables. |
| `009e5580` | `FactoryPacketListToolsResponse_ctor` | ✅ | ctor | Constructor of FactoryPacketListToolsResponse: runs the BaseFactoryPacket base constructor with sub-opcode 0x16, stores the FactoryPacketListToolsResponse vtable and constructs the embedded tool-definition map with 009e… |
| `009e55e0` | `FactoryPacketListToolsResponse_scalarDeletingDtor` | 🟢 | dtor | Scalar deleting destructor: calls the destructor body FUN_009e41b0 (which destroys the embedded tool map via 009e4140 and resets the packet vtables) and frees this when bit 0 of the flag byte is set. |
