# Class: TradeConfirmPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c6b240` | `TradeConfirmPacket::TradeConfirmPacket` | ✅ | ctor | Constructor used by the trade dispatcher for the final confirm: calls the family constructor FUN_00c6a7b0(0x12), installs TradeConfirmPacket::vftable and stores one dword at +0xc. |
| `00c6bad0` | `TradeConfirmPacket::ScalarDeletingDtor` | ✅ | virtual | Scalar deleting destructor at slot 0 of the class vtable: stores BasePacket::vftable at +0 and calls _free on this when bit 0 of the flag is set. |
| `00c6c5f0` | `TradeConfirmPacket::ReadFields ⚠downgrade` | 🟢 | method | Field reader of the confirm packet: after the shared header read it reads one 32-bit value into the destination's +0xc with the standard short-read handling. |
| `00c6c9d0` | `— ⚠reject` | ⚪ | method | Deserialize entry point for the confirm packet: builds the reader frame over (data,size), calls the single-u32 reader 00c6c5f0 and returns 1 when no overflow occurred and the buffer was consumed exactly (or the strictne… |
