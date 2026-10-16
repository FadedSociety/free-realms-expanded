# Class: ClientHousingPacketToggleLocked

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa4d70` | `ClientHousingPacketToggleLocked::ClientHousingPacketToggleLocked` | ✅ | ctor | Calls FUN_00aa4990(0xb) to build the family 127 header with sub-opcode 11, then stores ClientHousingPacketToggleLocked::vftable. |
| `00aa7d50` | `ClientHousingPacketToggleLocked::ScalarDeletingDtor` | ✅ | dtor | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |
| `00ac0b90` | `SendClientHousingPacketToggleLocked` | 🟢 | function | Transport wrapper for the payloadless 127/11 lock toggle: opens the scratch scope with FUN_008d4a40(0x10000000), emits only the family header with FUN_00aba910 over the caller's packet, wraps the buffer in a tunneled Cl… |
