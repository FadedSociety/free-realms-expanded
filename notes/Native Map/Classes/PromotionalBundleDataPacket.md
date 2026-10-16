# Class: PromotionalBundleDataPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a05a50` | `PromotionalBundleDataPacket::PromotionalBundleDataPacket` | ✅ | ctor | Constructor: installs PromotionalBundleDataPacket::vftable at offset 0, calls FUN_00a057e0(2) which runs the BasePacket + BasePromotionalPacket base constructor and stores base opcode 0x72 and sub-opcode 2, then zeroes … |
| `00a05ab0` | `PromotionalBundleDataPacket::~PromotionalBundleDataPacket` | ✅ | dtor | Destructor body: rewrites the object vtable to PromotionalBundleDataPacket::vftable, then to BasePromotionalPacket::vftable, then to BasePacket::vftable; no members are released. |
| `00a05b10` | `PromotionalBundleDataPacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: runs the vtable-reset destructor chain (PromotionalBundleDataPacket, BasePromotionalPacket, BasePacket) and then calls _free(this) when the delete flag byte is set. |
