# Class: ClientHousingPacketSetEditMode

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa4cc0` | `ClientHousingPacketSetEditMode::ClientHousingPacketSetEditMode` | ✅ | ctor | Calls FUN_00aa4990(6) to build the family 127 header with sub-opcode 6, then stores ClientHousingPacketSetEditMode::vftable. |
| `00aa7ce0` | `ClientHousingPacketSetEditMode::ScalarDeletingDtor` | ✅ | dtor | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |
| `00abcb30` | `ClientHousingPacketSetEditMode::WriteBody` | ✅ | method | Thiscall on the edit-mode packet: writes the framing through FUN_00aba910, then one u8 from +0xc (the edit-mode flag) into the writer argument. |
| `00ac02f0` | `SendClientHousingPacketSetEditMode` | ✅ | function | Transport wrapper for 127/6: opens the scratch scope with FUN_008d4a40(0x10000000), runs the edit-mode body writer FUN_00abcb30 over the caller's packet (a single u8), wraps the buffer in a tunneled ClientGateway packet… |
