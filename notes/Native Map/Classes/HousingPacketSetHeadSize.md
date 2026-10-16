# Class: HousingPacketSetHeadSize

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa57b0` | `HousingPacketSetHeadSize::HousingPacketSetHeadSize` | ✅ | ctor | Calls FUN_00aa4990(0x32) to build the family 127 header with sub-opcode 50, stores HousingPacketSetHeadSize::vftable and sets the dword at object+0xc to 10. |
| `00aa87c0` | `HousingPacketSetHeadSize::ScalarDeletingDtor` | ✅ | dtor | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |
