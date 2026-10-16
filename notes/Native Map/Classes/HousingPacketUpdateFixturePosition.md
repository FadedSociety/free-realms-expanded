# Class: HousingPacketUpdateFixturePosition

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa5860` | `HousingPacketUpdateFixturePosition::~HousingPacketUpdateFixturePosition` | ✅ | dtor | Stores HousingPacketUpdateFixturePosition::vftable, then BaseHousingPacket::vftable, then BasePacket::vftable into the object. |
| `00aa58c0` | `HousingPacketUpdateFixturePosition::ScalarDeletingDtor` | ✅ | dtor | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |
| `00aa8830` | `HousingPacketUpdateFixturePosition::HousingPacketUpdateFixturePosition` | ✅ | ctor | Calls FUN_00aa4990(0x33) to build the family 127 header with sub-opcode 51, stores HousingPacketUpdateFixturePosition::vftable and copies two groups of four default dwords (from DAT_01be7bc8/bcc and the 0x1cb53xx and 0x… |
