# Class: InventoryPacketEquipLightsaber

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bda5c0` | `InventoryPacketEquipLightsaber_Ctor` | 🟢 | ctor | Constructs the packet with FUN_00a10400(0xd) (inventory family 42, sub-opcode 13), installs InventoryPacketEquipLightsaber::vftable and calls FUN_0101cd40 to initialise the packet's fixed-size string member. |
| `00bda620` | `InventoryPacketEquipLightsaber::~InventoryPacketEquipLightsaber` | 🟢 | dtor | Destructor body of the equip-lightsaber scratch: stores BaseInventoryPacket::vftable and then BasePacket::vftable at object+0, with no member cleanup. |
| `00bda8f0` | `InventoryPacketEquipLightsaber::ScalarDeletingDtor` | ✅ | virtual | Virtual scalar deleting destructor (slot 0 of InventoryPacketEquipLightsaber::vftable): stores BaseInventoryPacket::vftable then BasePacket::vftable at object+0 and frees the object through the 00d82463 _free thunk when… |
