# Class: ClientHousingPacketPlaceFixtureRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa5340` | `ClientHousingPacketPlaceFixtureRequest::ClientHousingPacketPlaceFixtureRequest` | ✅ | ctor | Calls FUN_00aa4990(1) to build family 127 with inner subtype 1, stores ClientHousingPacketPlaceFixtureRequest::vftable, then zeroes the embedded dword at +0xc through FUN_0101cd40 with ECX=this+0xc and no stack argument. |
| `00aa82e0` | `ClientHousingPacketPlaceFixtureRequest::ScalarDeletingDtor` | ✅ | dtor | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |
| `00abcb90` | `ClientHousingPacketPlaceFixtureRequest_SerializeBody` | ✅ | function | Calls FUN_00aba910 to write the housing header, then writes dwords from source+0xc, +0x10 and +0x14 and one byte from +0x18 through bounded writer FUN_007cc400, advancing the cursor by the accepted byte count. |
| `00ac0400` | `— ⚠reject` | ⚪ | function | Transport wrapper for 127/1: opens the scratch scope with FUN_008d4a40(0x10000000), runs the place-fixture body writer FUN_00abcb90 over the caller's packet (three u32 plus one byte), then wraps the produced buffer in a… |
