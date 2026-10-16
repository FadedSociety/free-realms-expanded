# Class: TradeCoinCountPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c6ad60` | `TradeCoinCountPacket::TradeCoinCountPacket` | ✅ | ctor | Constructor of the client's coin offer: calls the family constructor FUN_00c6a7b0(10), installs TradeCoinCountPacket::vftable and stores the session id at +0xc and the offered coin count at +0x10. |
| `00c6b930` | `TradeCoinCountPacket::ScalarDeletingDtor` | ✅ | virtual | Scalar deleting destructor at slot 0 of the class vtable: stores BasePacket::vftable at +0 and calls _free on this when bit 0 of the flag is set. |
| `00c6d240` | `TradeCoinCountPacket::Write` | 🟢 | method | Serialises the coin offer: writes the header with 00c6cc30, then copies the 32-bit session id at +0xc and the 32-bit coin count at +0x10 into the writer through FUN_007cc400. |
