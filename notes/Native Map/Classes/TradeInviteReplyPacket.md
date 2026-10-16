# Class: TradeInviteReplyPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c6a8a0` | `TradeInviteReplyPacket::TradeInviteReplyPacket` | ✅ | ctor | Constructor of the client's answer to a trade invitation: calls the family constructor FUN_00c6a7b0(3), installs TradeInviteReplyPacket::vftable, stores the session id at +0xc and the accepted flag as a byte at +0x10. |
| `00c6b6d0` | `TradeInviteReplyPacket::ScalarDeletingDtor` | ✅ | virtual | Scalar deleting destructor at slot 0 of the class vtable: stores BasePacket::vftable at +0 and calls _free on this when bit 0 of the flag is set. |
| `00c6cee0` | `TradeInviteReplyPacket::Write` | 🟢 | method | Serialises the invite reply into an outgoing buffer: writes the family and sub-opcode header with 00c6cc30, then copies the 32-bit session id at +0xc and the one-byte accepted flag at +0x10 into the writer through FUN_0… |
