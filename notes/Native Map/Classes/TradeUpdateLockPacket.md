# Class: TradeUpdateLockPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c6b300` | `TradeUpdateLockPacket::TradeUpdateLockPacket` | ✅ | ctor | Constructor used by the trade dispatcher for a lock-state update: calls the family constructor FUN_00c6a7b0(0x11), installs TradeUpdateLockPacket::vftable, stores the session id at +0xc and the locked flag as a byte at … |
| `00c6bb30` | `TradeUpdateLockPacket::ScalarDeletingDtor` | ✅ | virtual | Scalar deleting destructor at slot 0 of the class vtable: stores BasePacket::vftable at +0 and calls _free on this when bit 0 of the flag is set. |
| `00c6c110` | `TradeUpdateLockPacket::ReadFields` | ✅ | method | Field reader of the lock-update packet: calls the shared header reader 00c6beb0, then reads a 32-bit session id into this+0xc and a one-byte boolean lock flag into this+0x10 through the caller's reader, setting the over… |
| `00c6c8f0` | `TradeUpdateLockPacket::Read` | ✅ | method | Deserialize entry point for the lock-state update: builds the reader frame over (data,size), calls the u32-plus-bool reader 00c6c110 and returns 1 when no overflow occurred and the buffer was consumed exactly (or the st… |
