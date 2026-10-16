# Class: ClientHousingPacketDeclineInvite

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa7b10` | `ClientHousingPacketDeclineInvite::ClientHousingPacketDeclineInvite` | ✅ | ctor | Calls FUN_00aa4990(0x10) to build the family 127 header with sub-opcode 16, stores ClientHousingPacketDeclineInvite::vftable and copies two default dwords from DAT_01be7be0 and DAT_01be7be4 into object+0x10 and +0x14. |
| `00aa7b80` | `ClientHousingPacketDeclineInvite::ScalarDeletingDtor` | ✅ | dtor | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |
| `00ac1310` | `SendClientHousingPacketDeclineInvite` | 🟢 | function | Transport wrapper for 127/16: opens the scratch scope with FUN_008d4a40(0x10000000), emits the family header with FUN_00aba910, writes eight bytes from source+0x10 (the dwords at +0x10 and +0x14) through FUN_008fd1a0, w… |
