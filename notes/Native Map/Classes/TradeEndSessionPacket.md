# Class: TradeEndSessionPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c6a980` | `TradeEndSessionPacket::TradeEndSessionPacket ⚠downgrade` | 🟢 | ctor | Constructor used by the trade dispatcher for a received end-session message: calls the family constructor FUN_00c6a7b0(5), installs TradeEndSessionPacket::vftable and stores the session id at +0xc. |
| `00c6b750` | `TradeEndSessionPacket::ScalarDeletingDtor` | ✅ | virtual | Scalar deleting destructor at slot 0 of the class vtable: stores BasePacket::vftable at +0 and calls _free on this when bit 0 of the flag is set. |
| `00c6c530` | `TradeEndSessionPacket::ReadFields ⚠downgrade` | 🟢 | method | Field reader of the end-session packet: calls the shared header reader 00c6beb0 with the destination packet in ecx, then reads one 32-bit value into the destination's +0xc, zeroing it and setting the reader overflow fla… |
| `00c6c730` | `— ⚠reject` | ⚪ | method | Deserialize entry point for the end-session packet: when both the destination packet and the buffer are non-null it lays a reader frame over (param_2, param_2+param_3) with the overflow byte cleared, calls the field rea… |
