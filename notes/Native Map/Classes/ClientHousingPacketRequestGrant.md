# Class: ClientHousingPacketRequestGrant

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa4fd0` | `ClientHousingPacketRequestGrant::ClientHousingPacketRequestGrant` | ✅ | ctor | Calls FUN_00aa4990(0x17) to build the family 127 header with sub-opcode 23, then stores ClientHousingPacketRequestGrant::vftable. |
| `00aa7f80` | `ClientHousingPacketRequestGrant::ScalarDeletingDtor` | ✅ | dtor | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |
| `00ac1d30` | `SendClientHousingPacketRequestGrant` | 🟢 | function | Transport wrapper for the payloadless property-grant request: opens the scratch scope with FUN_008d4a40(0x10000000), emits only the family header with FUN_00aba910 over the caller's packet, wraps the buffer in a tunnele… |
