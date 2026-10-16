# Class: TradeUpdateAcceptPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c6b160` | `TradeUpdateAcceptPacket::TradeUpdateAcceptPacket ⚠downgrade` | 🟢 | ctor | Constructor used by the trade dispatcher for an accept-state update: calls the family constructor FUN_00c6a7b0(0xe), installs TradeUpdateAcceptPacket::vftable and stores one dword at +0xc. |
| `00c6ba70` | `TradeUpdateAcceptPacket::ScalarDeletingDtor` | ✅ | virtual | Scalar deleting destructor at slot 0 of the class vtable: stores BasePacket::vftable at +0 and calls _free on this when bit 0 of the flag is set. |
| `00c6c590` | `TradeUpdateAcceptPacket::ReadFields` | 🟢 | method | Field reader of the accept-state update: after the shared header read it reads one 32-bit value into the destination's +0xc, zeroing it and setting the overflow flag on a short read. |
| `00c6c880` | `— ⚠reject` | ⚪ | method | Deserialize entry point for the accept-state update: builds the reader frame over (data,size), calls the single-u32 reader 00c6c590 and returns 1 when no overflow occurred and the buffer was fully consumed (or the stric… |
