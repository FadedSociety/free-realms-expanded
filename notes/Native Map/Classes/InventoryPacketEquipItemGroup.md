# Class: InventoryPacketEquipItemGroup

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bda670` | `InventoryPacketEquipItemGroup_Ctor` | 🟢 | ctor | Constructs the packet with FUN_00a10400(0xe) (inventory family 42, sub-opcode 14) and installs InventoryPacketEquipItemGroup::vftable. |
| `00bda6d0` | `InventoryPacketEquipItemGroup::~InventoryPacketEquipItemGroup` | 🟢 | dtor | Destructor body of the equip-item-group scratch: stores BaseInventoryPacket::vftable and then BasePacket::vftable at object+0, with no member cleanup. |
| `00bda960` | `InventoryPacketEquipItemGroup::ScalarDeletingDtor` | ✅ | virtual | Virtual scalar deleting destructor (slot 0 of InventoryPacketEquipItemGroup::vftable): stores BaseInventoryPacket::vftable then BasePacket::vftable at object+0 and frees the object through the 00d82463 _free thunk when … |
