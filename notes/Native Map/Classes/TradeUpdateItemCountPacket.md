# Class: TradeUpdateItemCountPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c6b040` | `TradeUpdateItemCountPacket::TradeUpdateItemCountPacket` | ✅ | ctor | Constructor used by the trade dispatcher for an item-count update: calls the family constructor FUN_00c6a7b0(0xd), installs TradeUpdateItemCountPacket::vftable, initialises +0xc and +0x10 to 0xffffffff, the flag byte at… |
| `00c6ba10` | `TradeUpdateItemCountPacket::ScalarDeletingDtor` | ✅ | virtual | Scalar deleting destructor at slot 0 of the class vtable: stores BasePacket::vftable at +0 and calls _free on this when bit 0 of the flag is set. |
| `00c6c000` | `— ⚠reject` | ⚪ | method | Field reader of the item-count update packet: after the shared header read it reads a 32-bit session id into +0xc, a 32-bit count into +0x10, a byte boolean into +0x14, then a 32-bit item id into +0x18 and a 32-bit coun… |
| `00c6c810` | `— ⚠reject` | ⚪ | method | Deserialize entry point for the item-count update: builds the reader frame over (data,size), calls the five-field reader 00c6c000 and returns 1 when no overflow occurred and the buffer was consumed exactly (or the stric… |
