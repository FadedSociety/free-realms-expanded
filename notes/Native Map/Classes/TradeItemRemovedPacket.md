# Class: TradeItemRemovedPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c6b3e0` | `TradeItemRemovedPacket::TradeItemRemovedPacket` | ✅ | ctor | Constructor used by the trade dispatcher for an item removed from an offer: calls the family constructor FUN_00c6a7b0(0x13), installs TradeItemRemovedPacket::vftable and stores three dwords at +0xc, +0x10 and +0x14. |
| `00c6bb90` | `TradeItemRemovedPacket::ScalarDeletingDtor` | ✅ | virtual | Scalar deleting destructor at slot 0 of the class vtable: stores BasePacket::vftable at +0 and calls _free on this when bit 0 of the flag is set. |
| `00c6c180` | `TradeItemRemovedPacket::ReadFields` | ✅ | method | Field reader of the item-removed packet: after the shared header read it reads three consecutive 32-bit values into this+0xc, this+0x10 and this+0x14, each with its own bounds check that zeroes the field, sets the reade… |
| `00c6c960` | `— ⚠reject` | ⚪ | method | Deserialize entry point for the item-removed packet: builds the reader frame over (data,size), calls the three-u32 reader 00c6c180 and returns 1 when no overflow occurred and the buffer was consumed exactly (or the stri… |
