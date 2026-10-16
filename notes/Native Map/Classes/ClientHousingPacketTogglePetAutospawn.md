# Class: ClientHousingPacketTogglePetAutospawn

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa4f20` | `ClientHousingPacketTogglePetAutospawn::ClientHousingPacketTogglePetAutospawn` | ✅ | ctor | Calls FUN_00aa4990(0xd) to build the family 127 header with sub-opcode 13, then stores ClientHousingPacketTogglePetAutospawn::vftable. |
| `00aa7f10` | `ClientHousingPacketTogglePetAutospawn::ScalarDeletingDtor` | ✅ | dtor | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |
| `00ac0ec0` | `SendClientHousingPacketTogglePetAutospawn` | 🟢 | function | Transport wrapper for the payloadless 127/13 pet-autospawn toggle: opens the scratch scope with FUN_008d4a40(0x10000000), emits only the family header with FUN_00aba910 over the caller's packet, wraps the buffer in a tu… |
