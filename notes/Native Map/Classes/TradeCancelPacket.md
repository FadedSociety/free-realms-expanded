# Class: TradeCancelPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c6abd0` | `TradeCancelPacket::TradeCancelPacket` | ✅ | ctor | Constructor of the client's cancel request: calls the family constructor FUN_00c6a7b0(7), installs TradeCancelPacket::vftable and stores the session id at +0xc. |
| `00c6b870` | `TradeCancelPacket::ScalarDeletingDtor` | ✅ | virtual | Scalar deleting destructor at slot 0 of the class vtable: stores BasePacket::vftable at +0 and calls _free on this when bit 0 of the flag is set. |
| `00c6d100` | `— ⚠reject` | ⚪ | method | Serialises the cancel request: writes the header with 00c6cc30 and copies the single 32-bit session id at +0xc into the writer through FUN_007cc400. |
