# Class: TradeInvitePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c6c290` | `TradeInvitePacket::TradeInvitePacket ⚠downgrade` | 🟢 | ctor | Constructor used by the trade dispatcher for an incoming invitation: calls the family constructor FUN_00c6a7b0(2), installs TradeInvitePacket::vftable, constructs the string member at +0xc with FUN_0070bac0, copies the … |
| `00c6c320` | `TradeInvitePacket::ScalarDeletingDtor` | ✅ | virtual | Scalar deleting destructor at slot 0 of the class vtable: calls the string-carrying destructor body 00c6bd10 (which releases the member at +0xc and restores the base vtable) and calls _free on this when bit 0 of the fla… |
| `00c6cab0` | `TradeInvitePacket::ReadFields ? ⚠downgrade` | 🟡 | method | Field reader of the invite packet: reads the family/sub-opcode header with 00c6beb0, then reads a length-prefixed string into the member at +0xc through FUN_008e7290, then two 32-bit values into +0x98 and +0x9c in one b… |
| `00c6ccd0` | `— ⚠reject` | ⚪ | method | Deserialize entry point for the invite packet: builds the reader frame over (data,size), calls the invite field reader 00c6cab0 and returns 1 when no overflow occurred and the buffer was consumed exactly (or the strictn… |
