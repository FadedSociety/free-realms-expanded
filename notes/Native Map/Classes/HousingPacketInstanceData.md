# Class: HousingPacketInstanceData

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa55f0` | `HousingPacketInstanceData::HousingPacketInstanceData` | ✅ | ctor | Calls FUN_00aa4990(0x1c) to build the family 127 header with sub-opcode 28, stores HousingPacketInstanceData::vftable and writes the constructor argument into object+0xc. |
| `00aa85f0` | `HousingPacketInstanceData::ScalarDeletingDtor` | ✅ | dtor | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |
