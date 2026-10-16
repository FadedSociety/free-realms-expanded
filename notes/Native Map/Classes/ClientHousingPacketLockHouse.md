# Class: ClientHousingPacketLockHouse

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa7dc0` | `ClientHousingPacketLockHouse::ClientHousingPacketLockHouse` | ✅ | ctor | Calls FUN_00aa4990(0x16) to build the family 127 header with sub-opcode 22, stores ClientHousingPacketLockHouse::vftable, copies two default dwords from DAT_01be7be0 and DAT_01be7be4 and clears the flag byte at object+0… |
| `00aa7e30` | `ClientHousingPacketLockHouse::ScalarDeletingDtor` | ✅ | dtor | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |
| `00abce00` | `ClientHousingPacketLockHouse_SerializeBody` | 🟢 | function | Calls FUN_00aba910 to write the housing header, serializes eight bytes beginning at source+0x10 through FUN_008fd1a0, then writes the byte at source+0x18 through FUN_007cc400. |
| `00ac0ca0` | `SendClientHousingPacketLockHouse` | 🟢 | function | Transport wrapper for 127/22: opens the scratch scope with FUN_008d4a40(0x10000000), runs the body writer FUN_00abce00 over the caller's packet (one u64 and one byte), wraps the produced buffer in a tunneled ClientGatew… |
