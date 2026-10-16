# Class: InventoryPacketUseStyleCard

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c10890` | `InventoryPacketUseStyleCard::InventoryPacketUseStyleCard` | ✅ | ctor | Constructor: calls the inventory-packet base constructor FUN_00a10400 with sub-opcode 10 (0xa), stores InventoryPacketUseStyleCard::vftable at *this and returns the object (the stack-cookie check runs through FUN_0101cd… |
| `00c108f0` | `InventoryPacketUseStyleCard::~InventoryPacketUseStyleCard` | 🟢 | dtor | Destructor body: resets the object's vtables to BaseInventoryPacket::vftable and then BasePacket::vftable and returns (all members are scalar, no releases). |
| `00c11240` | `InventoryPacketUseStyleCard::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0 of InventoryPacketUseStyleCard): resets the vtables to BaseInventoryPacket::vftable and BasePacket::vftable and frees the object through _free when bit 0 of param_2 is set. |
