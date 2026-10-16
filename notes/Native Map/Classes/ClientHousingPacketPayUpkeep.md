# Class: ClientHousingPacketPayUpkeep

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa4b70` | `ClientHousingPacketPayUpkeep::ClientHousingPacketPayUpkeep` | ✅ | ctor | Calls FUN_00aa4990(7) to build the family 127 header with sub-opcode 7, then stores ClientHousingPacketPayUpkeep::vftable. |
| `00aa7aa0` | `ClientHousingPacketPayUpkeep::ScalarDeletingDtor` | ✅ | dtor | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |
| `00ac0970` | `SendClientHousingPacketPayUpkeep` | 🟢 | function | Transport wrapper for the payloadless 127/7 upkeep payment: opens the scratch scope with FUN_008d4a40(0x10000000), emits only the family header with FUN_00aba910 over the caller's packet, wraps the buffer in a tunneled … |
