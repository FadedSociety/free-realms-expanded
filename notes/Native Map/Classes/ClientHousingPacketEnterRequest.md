# Class: ClientHousingPacketEnterRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa83c0` | `ClientHousingPacketEnterRequest::ClientHousingPacketEnterRequest` | ✅ | ctor | Calls FUN_00aa4990(0x13) to build the family 127 header with sub-opcode 19, stores ClientHousingPacketEnterRequest::vftable, copies two default dwords from DAT_01be7be0 and DAT_01be7be4, and zeroes the dword at +0x18 an… |
| `00aa8430` | `ClientHousingPacketEnterRequest::ScalarDeletingDtor` | ✅ | dtor | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |
| `00abca80` | `ClientHousingPacketEnterRequest::WriteBody` | ✅ | method | Thiscall on the enter-request packet: writes the framing through FUN_00aba910, then a u64 from +0x10 through FUN_008fd1a0 (two u32 words in order), a u32 from +0x18 and a u8 from +0x1c into the writer argument. |
| `00ac01e0` | `— ⚠reject` | ⚪ | function | Transport wrapper for 127/19: opens the shared transport scratch scope with FUN_008d4a40(0x10000000), runs the enter-request body writer FUN_00abca80 over the caller's packet (the writer emits the u64, u32 and u8 body),… |
