# Class: ClientHousingPacketExplosionResetRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa5a30` | `ClientHousingPacketExplosionResetRequest::ClientHousingPacketExplosionResetRequ…` | ✅ | ctor | Calls FUN_00aa4990(0x12) to build the family 127 header with sub-opcode 18, then stores ClientHousingPacketExplosionResetRequest::vftable. |
| `00aa8a00` | `ClientHousingPacketExplosionResetRequest::ScalarDeletingDtor` | ✅ | dtor | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |
| `00ac1e40` | `SendClientHousingPacketExplosionResetRequest` | 🟢 | function | Transport wrapper for the payloadless explosion-reset request: opens the scratch scope with FUN_008d4a40(0x10000000), emits only the family header with FUN_00aba910 over the caller's packet, wraps the buffer in a tunnel… |
