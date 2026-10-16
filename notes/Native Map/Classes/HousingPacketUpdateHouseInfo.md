# Class: HousingPacketUpdateHouseInfo

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa5080` | `HousingPacketUpdateHouseInfo::HousingPacketUpdateHouseInfo` | ✅ | ctor | Calls FUN_00aa4990(0x2c) to build the family 127 header with sub-opcode 44, stores HousingPacketUpdateHouseInfo::vftable, and sets the flag bytes at +0xc..+0xf to 0,0,1,1 and zeroes the dwords at +0x10..+0x1c. |
| `00aa7ff0` | `HousingPacketUpdateHouseInfo::ScalarDeletingDtor` | ✅ | dtor | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |
