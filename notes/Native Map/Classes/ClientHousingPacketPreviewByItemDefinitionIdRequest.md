# Class: ClientHousingPacketPreviewByItemDefinitionIdRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa54f0` | `ClientHousingPacketPreviewByItemDefinitionIdRequest::Ctor` | ✅ | ctor | Calls FUN_00aa4990(0x14) to build the family 127 header with sub-opcode 20, stores ClientHousingPacketPreviewByItemDefinitionIdRequest::vftable and zeroes the dword at object+0xc. |
| `00aa84a0` | `ClientHousingPacketPreviewByItemDefinitionIdRequest::ScalarDeletingDtor` | ✅ | dtor | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |
| `00abcd90` | `ClientHousingPacketPreviewByItemDefinitionIdRequest_SerializeBody` | ✅ | function | Calls FUN_00aba910 to write the housing header, then writes the dword at source+0xc through FUN_007cc400 and advances the bounded cursor. |
| `00ac0a80` | `SendClientHousingPacketPreviewByItemDefinitionIdRequest` | ✅ | function | Transport wrapper for 127/20: opens the scratch scope with FUN_008d4a40(0x10000000), runs the body writer FUN_00abcd90 over the caller's packet (a single u32 item-definition id), wraps the buffer in a tunneled ClientGat… |
