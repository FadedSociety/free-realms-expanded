# Class: CoinStoreMerchantListPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c6ef90` | `CoinStoreMerchantListPacket::Ctor` | ✅ | ctor | Calls the base constructor with sub-opcode 10, installs CoinStoreMerchantListPacket::vftable, zeroes the word at +0xc, returns this. |
| `00c700f0` | `CoinStoreMerchantListPacket::ScalarDeletingDtor` | ✅ | dtor | Stores BaseCoinStorePacket and BasePacket vtables, then calls _free(this) through thunk 00d82463 when bit 0 of the second argument is set. Returns this. |
