# Class: CoinStoreSellToClientRequestPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c6ecf0` | `CoinStoreSellToClientRequestPacket::ItemRecordMemberDtor` | 🟢 | dtor | Stores ItemRecord::vftable into word [7] (offset +0x1c, an embedded ItemRecord), then resets the BaseCoinStorePacket and BasePacket vtables. No free and no callees. |
| `00c6fef0` | `CoinStoreSellToClientRequestPacket::Ctor` | ✅ | ctor | Calls the base constructor with sub-opcode 4, installs CoinStoreSellToClientRequestPacket::vftable, copies two words from globals DAT_01beea08/DAT_01beea0c into +0x10/+0x14, embeds ItemRecord::vftable at word [7] (+0x1c… |
| `00c6ff70` | `CoinStoreSellToClientRequestPacket::ScalarDeletingDtor` | ✅ | dtor | Calls 00c6ecf0 (embedded ItemRecord teardown), then calls _free(this) through thunk 00d82463 when bit 0 of the second argument is set. Returns this. |
