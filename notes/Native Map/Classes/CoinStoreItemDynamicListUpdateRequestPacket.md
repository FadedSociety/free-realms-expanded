# Class: CoinStoreItemDynamicListUpdateRequestPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c6ee60` | `CoinStoreItemDynamicListUpdateRequestPacket::Ctor` | ✅ | ctor | Calls the base constructor with sub-opcode 8, installs CoinStoreItemDynamicListUpdateRequestPacket::vftable, returns this. |
| `00c6eec0` | `CoinStoreItemDynamicListUpdateRequestPacket::Dtor` | ✅ | dtor | Installs the request, base and BasePacket vtables in turn; no free and no callees. |
| `00c6ef20` | `CoinStoreItemDynamicListUpdateRequestPacket::ScalarDeletingDtor` | ✅ | dtor | Reinstalls the request, base and BasePacket vtables, then calls _free(this) through thunk 00d82463 when bit 0 of the second argument is set. Returns this. |
