# Class: TradeUpdateCoinCountPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c6ae20` | `TradeUpdateCoinCountPacket::TradeUpdateCoinCountPacket` | ✅ | ctor | Constructor used by the trade dispatcher for a coin-count update: calls the family constructor FUN_00c6a7b0(0xb), installs TradeUpdateCoinCountPacket::vftable, stores the session id at +0xc, a count at +0x10, a byte fla… |
| `00c6b990` | `TradeUpdateCoinCountPacket::ScalarDeletingDtor` | ✅ | virtual | Scalar deleting destructor at slot 0 of the class vtable: stores BasePacket::vftable at +0 and calls _free on this when bit 0 of the flag is set. |
| `00c6bf50` | `TradeUpdateCoinCountPacket::ReadFields` | 🟢 | method | Field reader of the coin-count update packet: calls the shared header reader 00c6beb0 and then reads a 32-bit session id into this+0xc, a 32-bit count into this+0x10, a one-byte boolean into this+0x14 and a 32-bit value… |
| `00c6c7a0` | `— ⚠reject` | ⚪ | method | Deserialize entry point for the coin-count update: lays the same reader frame over (data,size), calls the field reader 00c6bf50 and returns 1 when no overflow was flagged and the buffer was fully consumed (or the strict… |
