# Class: InventoryPacketCheckImmediateActivationItem

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c109f0` | `— ⚠reject` | ⚪ | ctor | Constructor: calls the inventory-packet base constructor FUN_00a10400 with sub-opcode 0x10 (16), stores InventoryPacketCheckImmediateActivationItem::vftable at *this and returns the object. |
| `00c10a50` | `InventoryPacketCheckImmediateActivationItem::~InventoryPacketCheckImmediateActi…` | 🟢 | dtor | Destructor body: resets the object's vtables to BaseInventoryPacket::vftable and BasePacket::vftable and returns. |
| `00c11320` | `InventoryPacketCheckImmediateActivationItem::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0 of InventoryPacketCheckImmediateActivationItem): resets the vtables to BaseInventoryPacket::vftable and BasePacket::vftable and frees the object through _free when bit 0 of para… |
