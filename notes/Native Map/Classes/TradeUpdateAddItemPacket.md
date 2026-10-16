# Class: TradeUpdateAddItemPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c6af30` | `TradeUpdateAddItemPacket::TradeUpdateAddItemPacket` | ✅ | ctor | Constructor used by the trade dispatcher for an item-added update: calls the family constructor FUN_00c6a7b0(0xc), installs TradeUpdateAddItemPacket::vftable, initialises +0xc and +0x10 to 0xffffffff, the flag byte at +… |
| `00c6b9f0` | `TradeUpdateAddItemPacket::ScalarDeletingDtor` | ✅ | virtual | Scalar deleting destructor at slot 0 of the class vtable: calls the destructor body 00c6afa0 (which destroys the item member at +0x18 and restores BasePacket::vftable) and calls _free on this when bit 0 of the flag is s… |
| `00c6ce30` | `TradeUpdateAddItemPacket::ReadFields` | ✅ | method | Field reader of the add-item update: reads the header with 00c6beb0, then a 32-bit session id into +0xc, a 32-bit item value into +0x10, a byte boolean into +0x14, and finally passes the remaining bytes to FUN_008fa4b0 … |
| `00c6d350` | `— ⚠reject` | ⚪ | method | Deserialize entry point for the add-item update: builds the reader frame over (data,size), calls the add-item field reader 00c6ce30 (which reads u32, u32, bool and then the item member) and returns 1 when no overflow oc… |
