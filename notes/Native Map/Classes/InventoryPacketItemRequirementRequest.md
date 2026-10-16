# Class: InventoryPacketItemRequirementRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c08fa0` | `— ⚠reject` | ⚪ | ctor | Inventory packet constructor for subtype 4: calls the inventory base constructor FUN_00a10400(4), installs InventoryPacketItemRequirementRequest::vftable and initialises the embedded string/guid member at +0xc through F… |
| `00c096b0` | `InventoryPacketItemRequirementRequest_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor, slot 0 of InventoryPacketItemRequirementRequest::vftable: rewrites the vtable to BaseInventoryPacket::vftable then BasePacket::vftable and calls _free when bit 0 of the flag byte argument is … |
