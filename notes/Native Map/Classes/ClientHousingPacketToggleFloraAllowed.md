# Class: ClientHousingPacketToggleFloraAllowed

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa4e70` | `ClientHousingPacketToggleFloraAllowed::ClientHousingPacketToggleFloraAllowed` | ✅ | ctor | Calls FUN_00aa4990(0xc) to build the family 127 header with sub-opcode 12, then stores ClientHousingPacketToggleFloraAllowed::vftable. |
| `00aa7ea0` | `ClientHousingPacketToggleFloraAllowed::ScalarDeletingDtor` | ✅ | dtor | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |
| `00ac0db0` | `SendClientHousingPacketToggleFloraAllowed` | 🟢 | function | Transport wrapper for the payloadless 127/12 flora toggle: opens the scratch scope with FUN_008d4a40(0x10000000), emits only the family header with FUN_00aba910 over the caller's packet, wraps the buffer in a tunneled C… |
