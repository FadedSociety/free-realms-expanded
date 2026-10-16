# Class: CoinStoreBuyFromClientRequestPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c6ff90` | `— ⚠reject` | ⚪ | ctor | Calls the base constructor with sub-opcode 5, installs CoinStoreBuyFromClientRequestPacket::vftable, copies the two globals DAT_01beea08/DAT_01beea0c into +0x10/+0x14 around a stack-cookie check (call 0101cd40), zeroes … |
| `00c70010` | `CoinStoreBuyFromClientRequestPacket::ScalarDeletingDtor` | ✅ | dtor | Stores BaseCoinStorePacket and BasePacket vtables (the derived class's own vtable is not stored here), then calls _free(this) through thunk 00d82463 when bit 0 of the second argument is set. Returns this. |
