# Class: InventoryPacketItemActionBarAssign

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c14720` | `InventoryPacketItemActionBarAssign::InventoryPacketItemActionBarAssign` | ✅ | ctor | Constructor taking a slot number and a pointer to the item instance: calls the inventory base constructor FUN_00a10400 with sub-opcode 6, stores InventoryPacketItemActionBarAssign::vftable at *this, writes param_2 at +0… |
| `00c14790` | `InventoryPacketItemActionBarAssign::~InventoryPacketItemActionBarAssign` | 🟢 | dtor | Destructor body: resets the vtables to BaseInventoryPacket::vftable then BasePacket::vftable and returns. |
| `00c1a850` | `InventoryPacketItemActionBarAssign::ScalarDeletingDtor` | ✅ | dtor | Resets the inherited inventory and packet vptrs, then frees the allocation when the deleting flag bit is set. |
