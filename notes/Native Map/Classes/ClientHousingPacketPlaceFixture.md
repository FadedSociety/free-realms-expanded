# Class: ClientHousingPacketPlaceFixture

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa53f0` | `ClientHousingPacketPlaceFixture::ClientHousingPacketPlaceFixture` | ✅ | ctor | Calls FUN_00aa4990(2) to build family 127 with inner subtype 2, stores ClientHousingPacketPlaceFixture::vftable, then zeroes the embedded dword at +0xc through FUN_0101cd40 with ECX=this+0xc and no stack argument. |
| `00aa8350` | `ClientHousingPacketPlaceFixture::ScalarDeletingDtor` | ✅ | dtor | Stores BaseHousingPacket::vftable then BasePacket::vftable at the object start; frees this when flag bit 0 is set and otherwise returns this. |
| `00abc9a0` | `ClientHousingPacketPlaceFixture::WriteBody` | 🟢 | function | Thiscall on a ClientHousingPacketPlaceFixture object: writes the BaseHousingPacket framing, then a 4-byte value from +0xc, two 16-byte groups from +0x10 and +0x20 through FUN_0091a640, and a 4-byte value from +0x30. Raw… |
| `00abffb0` | `— ⚠reject` | ⚪ | function | Transport wrapper for the 127/2 place-fixture commit: opens the scratch scope with FUN_008d4a40(0x10000000), runs the body writer FUN_00abc9a0 over the caller's packet object, wraps the produced buffer in a tunneled Cli… |
