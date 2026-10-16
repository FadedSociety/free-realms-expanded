# Class: CoinStoreItemListPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c6f440` | `CoinStoreItemListPacket::Ctor` | ✅ | ctor | Calls the base constructor with sub-opcode 1, installs the CoinStoreItemListPacket<HashList<ItemDefinitionMetaData,4096,1024>,...> vtable, zeroes the words at +0xc and +0x10, returns this. |
| `00c6f4a0` | `CoinStoreItemListPacket::Dtor → CoinStoreItemListPacket<SoeUtil::HashList<ItemD…` | ✅ | dtor | Installs the item-list, BaseCoinStorePacket and BasePacket vtables in turn; no free and no callees. |
| `00c6f5c0` | `CoinStoreItemListPacket::ScalarDeletingDtor` | ✅ | dtor | Reinstalls the item-list, BaseCoinStorePacket and BasePacket vtables, then calls _free(this) through thunk 00d82463 when bit 0 of the second argument is set. Returns this. |
