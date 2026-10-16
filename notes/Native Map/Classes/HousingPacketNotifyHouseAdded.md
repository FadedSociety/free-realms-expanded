# Class: HousingPacketNotifyHouseAdded

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa86d0` | `HousingPacketNotifyHouseAdded::HousingPacketNotifyHouseAdded` | ✅ | ctor | Calls FUN_00aa4990(0x31) to build the family 127 header with sub-opcode 49, stores HousingPacketNotifyHouseAdded::vftable, zeroes the dword at +0xc and copies four default dwords into +0x10..+0x1c. |
| `00aa8750` | `HousingPacketNotifyHouseAdded::ScalarDeletingDtor` | ✅ | dtor | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |
