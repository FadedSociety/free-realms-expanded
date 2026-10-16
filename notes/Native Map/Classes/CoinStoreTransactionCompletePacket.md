# Class: CoinStoreTransactionCompletePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c6eda0` | `CoinStoreTransactionCompletePacket::Ctor` | ✅ | ctor | Calls the base constructor with sub-opcode 6, installs CoinStoreTransactionCompletePacket::vftable, sets word [3] to 2 and word [5] to 0 around a stack-cookie check (call 0101cd40), returns this. |
| `00c70080` | `CoinStoreTransactionCompletePacket::ScalarDeletingDtor` | ✅ | dtor | Stores BaseCoinStorePacket and BasePacket vtables, then calls _free(this) through thunk 00d82463 when bit 0 of the second argument is set. Returns this. |
