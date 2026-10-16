# Class: ClientHousingPacketRequestVisitToNeighbor

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa8510` | `ClientHousingPacketRequestVisitToNeighbor::ClientHousingPacketRequestVisitToNei…` | ✅ | ctor | Calls FUN_00aa4990(0x15) to build the family 127 header with sub-opcode 21, stores ClientHousingPacketRequestVisitToNeighbor::vftable and copies two default dwords from DAT_01be7bc8 and DAT_01be7bcc into object+0x10 and… |
| `00aa8580` | `ClientHousingPacketRequestVisitToNeighbor::ScalarDeletingDtor` | ✅ | dtor | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |
| `00ac1c10` | `SendClientHousingPacketRequestVisitToNeighbor` | 🟢 | function | Transport wrapper for 127/21: opens the scratch scope with FUN_008d4a40(0x10000000), emits the family header with FUN_00aba910 and appends the client's own guid pair at +0x10 through FUN_008fb810, wraps the buffer in a … |
