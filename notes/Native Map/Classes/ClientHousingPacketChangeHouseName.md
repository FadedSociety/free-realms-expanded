# Class: ClientHousingPacketChangeHouseName

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa5140` | `ClientHousingPacketChangeHouseName::ClientHousingPacketChangeHouseName` | ✅ | ctor | Calls FUN_00aa4990(9) to build the family 127 header with sub-opcode 9, then stores ClientHousingPacketChangeHouseName::vftable. |
| `00aa8060` | `ClientHousingPacketChangeHouseName::ScalarDeletingDtor` | ✅ | dtor | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |
| `00abce90` | `ClientHousingPacketChangeHouseName_SerializeBody` | 🟢 | function | Calls FUN_00aba910 for the housing header, then writes only the dword at source+0xc through FUN_007cc400; no string helper is called. |
| `00ac10e0` | `SendClientHousingPacketChangeHouseName` | 🟢 | function | Transport wrapper for 127/9: opens the scratch scope with FUN_008d4a40(0x10000000), runs the body writer FUN_00abce90 over the caller's packet (a single dword), wraps the produced buffer in a tunneled ClientGateway pack… |
