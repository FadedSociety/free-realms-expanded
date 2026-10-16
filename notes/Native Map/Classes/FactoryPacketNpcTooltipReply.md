# Class: FactoryPacketNpcTooltipReply

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009e40d0` | `FactoryPacketNpcTooltipReply::Dtor` | ✅ | dtor | Calls 009e2760 with ECX=this+0x18 to destroy the tooltip-effect list, then restores BaseFactoryPacket and BasePacket vtables. |
| `009e5500` | `FactoryPacketNpcTooltipReply_ctor` | ✅ | ctor | Constructor of FactoryPacketNpcTooltipReply: runs the BaseFactoryPacket base constructor with sub-opcode 0x15, stores the FactoryPacketNpcTooltipReply vtable and initialises the reply member through FUN_009e4000; return… |
| `009e5560` | `FactoryPacketNpcTooltipReply_scalarDeletingDtor` | 🟢 | dtor | Scalar deleting destructor: calls the destructor body FUN_009e40d0 and frees this through _free when bit 0 of the flag byte is set, returning this otherwise. |
