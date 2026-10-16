# Class: ReferenceDataPacketItemCategoryDefinitions

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a4740` | `ReferenceDataPacketItemCategoryDefinitions::ReferenceDataPacketItemCategoryDefi…` | ✅ | ctor | Item-category-definitions packet constructor taking a payload pointer: calls FUN_008a4550(2) so the packet is opcode 44 with sub-opcode 2, installs ReferenceDataPacketItemCategoryDefinitions::vftable and stores the payl… |
| `008a47b0` | `ReferenceDataPacketItemCategoryDefinitions::~ReferenceDataPacketItemCategoryDef…` | 🟢 | dtor | Destructor body: writes ReferenceDataPacket::vftable then BasePacket::vftable into the object vtable slot and returns; the 00928d00 dispatcher and EH funclet 01590028 call it. |
| `008b7d20` | `ReferenceDataPacketItemCategoryDefinitions::ScalarDeletingDtor` | ✅ | dtor | Restores ReferenceDataPacket::vftable then BasePacket::vftable, and frees the object when the deleting flag bit is set. |
