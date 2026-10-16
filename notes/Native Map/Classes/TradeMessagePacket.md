# Class: TradeMessagePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c6b4d0` | `TradeMessagePacket::TradeMessagePacket` | ✅ | ctor | Constructor used by the trade dispatcher for a trade status message: calls the family constructor FUN_00c6a7b0(0xf), installs TradeMessagePacket::vftable and stores one dword at +0xc. |
| `00c6bbf0` | `TradeMessagePacket::ScalarDeletingDtor` | ✅ | virtual | Scalar deleting destructor at slot 0 of the class vtable: stores BasePacket::vftable at +0 and calls _free on this when bit 0 of the flag is set. |
| `00c6c630` | `TradeMessagePacket::ReadFields` | 🟢 | method | Field reader of the trade status message: after the shared header read it reads exactly one byte (sign-extended) into the destination's +0xc, with the same zero/overflow handling as the u32 readers. |
| `00c6ca40` | `— ⚠reject` | ⚪ | method | Deserialize entry point for the trade status message: builds the reader frame over (data,size), calls the one-byte reader 00c6c630 and returns 1 when no overflow occurred and the buffer was consumed exactly (or the stri… |
