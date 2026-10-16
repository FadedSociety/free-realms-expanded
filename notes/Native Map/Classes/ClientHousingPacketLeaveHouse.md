# Class: ClientHousingPacketLeaveHouse

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa4ac0` | `ClientHousingPacketLeaveHouse::ClientHousingPacketLeaveHouse` | ✅ | ctor | Calls FUN_00aa4990(10) to build the family 127 header with sub-opcode 10, then stores ClientHousingPacketLeaveHouse::vftable. |
| `00aa7a30` | `ClientHousingPacketLeaveHouse::ScalarDeletingDtor` | ✅ | dtor | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |
| `00ac0fd0` | `SendClientHousingPacketLeaveHouse` | 🟢 | function | Transport wrapper for the payloadless 127/10 leave request: opens the scratch scope with FUN_008d4a40(0x10000000), emits only the family header with FUN_00aba910 over the caller's packet, wraps the buffer in a tunneled … |
