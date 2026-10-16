# Class: TradeAcceptPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c6ab10` | `TradeAcceptPacket::TradeAcceptPacket ⚠downgrade` | 🟢 | ctor | Constructor of the client's accept request: calls the family constructor FUN_00c6a7b0(6), installs TradeAcceptPacket::vftable, stores the session id at +0xc and the revision at +0x10. |
| `00c6b810` | `TradeAcceptPacket::ScalarDeletingDtor` | ✅ | virtual | Scalar deleting destructor at slot 0 of the class vtable: stores BasePacket::vftable at +0 and calls _free on this when bit 0 of the flag is set. |
| `00c6d060` | `TradeAcceptPacket::Write ? ⚠downgrade` | 🟡 | method | Serialises the accept request: writes the header with 00c6cc30, then copies the 32-bit session id at +0xc and the 32-bit second field at +0x10 into the writer through FUN_007cc400. |
