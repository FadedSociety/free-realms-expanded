# Class: BasePromotionalPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a057e0` | `BasePromotionalPacket::BasePromotionalPacket` | ✅ | ctor | Family constructor: stores BasePacket::vftable, writes the base opcode 0x72 (114) into the packet id slot param_1[1], stores BasePromotionalPacket::vftable and then stores the caller-supplied sub-opcode in param_1[2]. |
| `00a05850` | `BasePromotionalPacket::~BasePromotionalPacket` | 🟢 | dtor | Destructor body: rewrites this->vtable to BasePromotionalPacket::vftable and then BasePacket::vftable; called from nine unwind funclets. |
| `00a058a0` | `BasePromotionalPacket::ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor in an SEH frame: restores BasePromotionalPacket::vftable then BasePacket::vftable and frees this when bit 0 of the flag is set. |
