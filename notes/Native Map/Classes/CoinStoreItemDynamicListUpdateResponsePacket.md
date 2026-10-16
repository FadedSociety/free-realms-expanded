# Class: CoinStoreItemDynamicListUpdateResponsePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c6f500` | `CoinStoreItemDynamicListUpdateResponsePacket::Ctor` | ✅ | ctor | Calls the base constructor with sub-opcode 9, installs the CoinStoreItemDynamicListUpdateResponsePacket<HashList<ItemDefinitionMetaData,256,-1>> vtable, zeroes the word at +0xc, returns this. |
| `00c6f560` | `CoinStoreItemDynamicListUpdateResponsePacket::Dtor` | ✅ | dtor | Installs the update-response, BaseCoinStorePacket and BasePacket vtables in turn; no free and no callees. |
| `00c6f630` | `CoinStoreItemDynamicListUpdateResponsePacket::ScalarDeletingDtor` | ✅ | dtor | Reinstalls the update-response and BaseCoinStorePacket vtables, then BasePacket, then calls _free(this) through thunk 00d82463 when bit 0 of the second argument is set. Returns this. |
