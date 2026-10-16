# Class: CoinStoreItemDefinitionsResponsePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c6ebc0` | `CoinStoreItemDefinitionsResponsePacket::Ctor` | ✅ | ctor | Calls the base constructor with sub-opcode 3, installs CoinStoreItemDefinitionsResponsePacket::vftable, zeroes the word at +0xc and the byte at +0x10, returns this. |
| `00c6ec20` | `CoinStoreItemDefinitionsResponsePacket::Dtor` | ✅ | dtor | Installs CoinStoreItemDefinitionsResponsePacket, BaseCoinStorePacket and BasePacket vtables in turn; no free and no callees. |
| `00c6ec80` | `CoinStoreItemDefinitionsResponsePacket::ScalarDeletingDtor` | ✅ | dtor | Reinstalls the response, base and BasePacket vtables, then calls _free(this) through thunk 00d82463 when bit 0 of the second argument is set. Returns this. |
