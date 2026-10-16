# Class: ClientHousingPacketInvitePlayer

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa7bf0` | `ClientHousingPacketInvitePlayer::ClientHousingPacketInvitePlayer` | ✅ | ctor | Calls FUN_00aa4990(0xf) to build the family 127 header with sub-opcode 15, stores ClientHousingPacketInvitePlayer::vftable and copies four default dwords from DAT_01be7bc8, DAT_01be7bcc, DAT_01be7be0 and DAT_01be7be4 in… |
| `00aa7c70` | `ClientHousingPacketInvitePlayer::ScalarDeletingDtor` | ✅ | dtor | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |
| `00ac11f0` | `SendClientHousingPacketInvitePlayer` | 🟢 | function | Transport wrapper for 127/15: opens the scratch scope with FUN_008d4a40(0x10000000), emits the family header with FUN_00aba910, writes the 64-bit field at +0x10 through FUN_008fb810 and the 64-bit field at +0x18 through… |
