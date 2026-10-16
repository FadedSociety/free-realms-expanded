# Class: BaseTradePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c6a7b0` | `— ⚠reject` | ⚪ | ctor | Family constructor for every trade message: installs BasePacket::vftable at +0, writes the family byte 0x3b (59) at +4, then swaps in BaseTradePacket::vftable and stores the sub-opcode argument at +8. All 18 trade sub-o… |
| `00c6b650` | `BaseTradePacket::ScalarDeletingDtor` | ✅ | virtual | Scalar deleting destructor at slot 0 of the BaseTradePacket vtable: under an SEH frame it writes BasePacket::vftable at +0 and calls _free on this when bit 0 of the flag is set. |
