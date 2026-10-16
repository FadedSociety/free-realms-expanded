# Class: TradeOfferItemPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c6ac90` | `TradeOfferItemPacket::TradeOfferItemPacket` | ✅ | ctor | Constructor of the client's offer change: calls the family constructor FUN_00c6a7b0(9), installs TradeOfferItemPacket::vftable, stores the session id at +0xc, copies the item pointer argument's first dword into +0x10 (o… |
| `00c6b8d0` | `TradeOfferItemPacket::ScalarDeletingDtor` | ✅ | virtual | Scalar deleting destructor at slot 0 of the class vtable: stores BasePacket::vftable at +0 and calls _free on this when bit 0 of the flag is set. |
| `00c6d160` | `TradeOfferItemPacket::Write` | 🟢 | method | Serialises the offer change: writes the header with 00c6cc30, then copies three 32-bit values - the session id at +0xc, the item value at +0x10 and the resulting count at +0x14 - into the writer through FUN_007cc400. |
