# Class: HousingPacketExplosionReset

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa5980` | `HousingPacketExplosionReset::HousingPacketExplosionReset` | ✅ | ctor | Calls FUN_00aa4990(0x38) to build the family 127 header with sub-opcode 56, then stores HousingPacketExplosionReset::vftable. |
| `00aa8990` | `HousingPacketExplosionReset::ScalarDeletingDtor` | ✅ | dtor | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |
