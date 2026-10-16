# Class: CoinStoreItemDefinitionsRequestPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c6ea90` | `CoinStoreItemDefinitionsRequestPacket::Ctor` | ✅ | ctor | Calls the base constructor with sub-opcode 2, installs CoinStoreItemDefinitionsRequestPacket::vftable, zeroes the word at +0xc, returns this. |
| `00c6eaf0` | `CoinStoreItemDefinitionsRequestPacket::Dtor` | ✅ | dtor | Installs CoinStoreItemDefinitionsRequestPacket, BaseCoinStorePacket and BasePacket vtables in turn; no free and no callees. |
| `00c6eb50` | `CoinStoreItemDefinitionsRequestPacket::ScalarDeletingDtor` | ✅ | dtor | Reinstalls the request, base and BasePacket vtables, then calls _free(this) through thunk 00d82463 when bit 0 of the second argument is set. Returns this. |
