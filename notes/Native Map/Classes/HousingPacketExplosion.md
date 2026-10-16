# Class: HousingPacketExplosion

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa88c0` | `HousingPacketExplosion::HousingPacketExplosion` | ✅ | ctor | Calls FUN_00aa4990(0x37) to build the family 127 header with sub-opcode 55, then stores HousingPacketExplosion::vftable. |
| `00aa8920` | `HousingPacketExplosion::ScalarDeletingDtor` | ✅ | dtor | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |
