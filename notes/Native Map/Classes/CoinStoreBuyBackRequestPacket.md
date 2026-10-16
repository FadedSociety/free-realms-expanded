# Class: CoinStoreBuyBackRequestPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c701d0` | `CoinStoreBuyBackRequestPacket::Ctor` | ✅ | ctor | Calls the base constructor with sub-opcode 0xc (12), installs CoinStoreBuyBackRequestPacket::vftable, copies a global (DAT_01beea20) into word [3], zeroes word [4]. Returns this. |
| `00c70240` | `CoinStoreBuyBackRequestPacket::ScalarDeletingDtor` | ✅ | dtor | Stores BaseCoinStorePacket and BasePacket vtables, then calls _free(this) through thunk 00d82463 when bit 0 of the second argument is set. Returns this. |
