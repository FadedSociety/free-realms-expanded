# Class: InventoryPacketEquipByGuid

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bda510` | `— ⚠reject` | ⚪ | ctor | Constructs the packet with FUN_00a10400(3) (inventory family 42, sub-opcode 3), installs InventoryPacketEquipByGuid::vftable and then calls FUN_0101cd40 to initialise the packet's fixed-size string member. |
| `00bda570` | `InventoryPacketEquipByGuid::~InventoryPacketEquipByGuid` | 🟢 | dtor | Destructor body of the equip-by-guid scratch: stores BaseInventoryPacket::vftable and then BasePacket::vftable at object+0, performing no member cleanup. |
| `00bda880` | `InventoryPacketEquipByGuid::ScalarDeletingDtor` | ✅ | virtual | Virtual scalar deleting destructor (slot 0 of InventoryPacketEquipByGuid::vftable): stores BaseInventoryPacket::vftable then BasePacket::vftable at object+0 and frees the object through the 00d82463 _free thunk when the… |
