# Class: ClientHousingPacketPickupFixture

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa81a0` | `ClientHousingPacketPickupFixture::ClientHousingPacketPickupFixture` | ✅ | ctor | Calls FUN_00aa4990(3) to build the family 127 header with sub-opcode 3, then stores ClientHousingPacketPickupFixture::vftable. |
| `00aa8200` | `ClientHousingPacketPickupFixture::ScalarDeletingDtor` | ✅ | dtor | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |
| `00ac0510` | `SendClientHousingPacketPickupFixture` | ✅ | function | Transport wrapper for 127/3: opens the scratch scope with FUN_008d4a40(0x10000000), emits the housing family header with FUN_00aba910 and writes the 64-bit fixture identity at +0x10 through the u64 emitter FUN_008fd1a0,… |
