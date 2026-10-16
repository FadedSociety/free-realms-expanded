# Class: TradeRejectedPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c6c3f0` | `TradeRejectedPacket::TradeRejectedPacket` | ✅ | ctor | Constructor used by the trade dispatcher for a rejected trade: calls the family constructor FUN_00c6a7b0(0x10), installs TradeRejectedPacket::vftable under an SEH frame and constructs the string member at +0xc with FUN_… |
| `00c6c450` | `TradeRejectedPacket::ScalarDeletingDtor` | ✅ | virtual | Scalar deleting destructor at slot 0 of the class vtable: calls the string-carrying destructor body 00c6bdd0 (member at +0xc) and calls _free on this when bit 0 of the flag is set. |
| `00c6cdb0` | `— ⚠reject` | ⚪ | method | Deserialize entry point for the rejected packet: when the packet and buffer are non-null it reads the family/sub-opcode header with 00c6beb0 and then the remaining payload as a length-prefixed string through FUN_008e729… |
