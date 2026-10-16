# Class: TradeLockPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c6aa50` | `TradeLockPacket::TradeLockPacket` | ✅ | ctor | Constructor of the client's lock request: calls the family constructor FUN_00c6a7b0(8), installs TradeLockPacket::vftable, stores the session id at +0xc, the offer revision at +0x10 and the locked flag as a byte at +0x1… |
| `00c6b7b0` | `TradeLockPacket::ScalarDeletingDtor` | ✅ | virtual | Scalar deleting destructor at slot 0 of the class vtable: stores BasePacket::vftable at +0 and calls _free on this when bit 0 of the flag is set. |
| `00c6cf80` | `TradeLockPacket::Write ? ⚠downgrade` | 🟡 | method | Serialises the lock request: writes the header with 00c6cc30, then copies the 32-bit session id at +0xc, the 32-bit revision at +0x10 and the one-byte locked flag at +0x14 into the writer through FUN_007cc400. |
