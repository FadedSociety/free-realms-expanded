# Class: ClientHousingPacketPickupAllFixtures

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa5290` | `ClientHousingPacketPickupAllFixtures::ClientHousingPacketPickupAllFixtures` | ✅ | ctor | Calls FUN_00aa4990(4) to build the family 127 header with sub-opcode 4, then stores ClientHousingPacketPickupAllFixtures::vftable. |
| `00aa8270` | `ClientHousingPacketPickupAllFixtures::ScalarDeletingDtor` | ✅ | dtor | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |
| `00ac0630` | `SendClientHousingPacketPickupAllFixtures ? ⚠downgrade` | 🟡 | function | Transport wrapper for a payloadless housing request: opens the scratch scope with FUN_008d4a40(0x10000000), emits only the family header with FUN_00aba910 over the caller's packet object (this writer adds no body fields… |
