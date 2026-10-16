# Class: InventoryPacketMoveItemToStorage

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c147e0` | `InventoryPacketMoveItemToStorage::~InventoryPacketMoveItemToStorage` | ✅ | dtor | Destructor body: stores InventoryPacketMoveItemToStorage::vftable, then BaseInventoryPacket::vftable and finally BasePacket::vftable at *this and returns. |
| `00c14840` | `InventoryPacketMoveItemToStorage::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0 of InventoryPacketMoveItemToStorage): resets the three vtables (own, BaseInventoryPacket, BasePacket) and frees the object through _free when bit 0 of param_2 is set. |
| `00c1a8c0` | `— ⚠reject` | ⚪ | ctor | Calls the base inventory packet constructor with sub-opcode 0x11, installs the packet vtable, then copies two words from one input and a word plus argument into fields. |
