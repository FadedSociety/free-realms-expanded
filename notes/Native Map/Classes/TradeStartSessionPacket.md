# Class: TradeStartSessionPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c6c340` | `TradeStartSessionPacket::TradeStartSessionPacket` | ✅ | ctor | Constructor used by the trade dispatcher when a session actually starts: calls the family constructor FUN_00c6a7b0(4), installs TradeStartSessionPacket::vftable, copies the global default pair DAT_01bee968/DAT_01bee96c … |
| `00c6c3d0` | `TradeStartSessionPacket::ScalarDeletingDtor` | ✅ | virtual | Scalar deleting destructor at slot 0 of the class vtable: calls the destructor body 00c6bd70 (string member at +0x18) and calls _free on this when bit 0 of the flag is set. |
| `00c6cb40` | `TradeStartSessionPacket::ReadFields` | 🟢 | method | Field reader of the start-session packet: reads the header with 00c6beb0, then two 32-bit identity values into +0x10 and +0x14 as one bounds-checked pair, then a length-prefixed string into the member at +0x18 through F… |
| `00c6cd40` | `TradeStartSessionPacket::Read` | 🟢 | method | Deserialize entry point for the start-session packet: builds the reader frame over (data,size), calls the start-session field reader 00c6cb40 and returns 1 when no overflow occurred and the buffer was consumed exactly (… |
