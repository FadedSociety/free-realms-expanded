# Class: SoeUtil::HashList<TextureAssetTrackingNode,128,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a887f0` | `HashListTrackingNodeVSlot1Alloc40 ?` | 🟡 | virtual | Virtual slot 1 (+4) of HashList<TextureAssetTrackingNode,128,-1>: calls 006fb9c0, then slot +4 of the object returned by 006fb860 when non-null, else slot +4 of DAT_01bdf280 when non-null, else calls 006fb950(0x28,4); r… |
| `00a88830` | `HashListTrackingNodeVSlot2Free4` | 🟢 | virtual | Virtual slot 2 (+8) of HashList<TextureAssetTrackingNode,128,-1>: pushes alignment 4 and its pointer argument, calls 00401c00 (free through the memory manager), and returns with ret 4. |
| `00a912c0` | `TextureTrackingHashListConstructor` | ✅ | ctor | Installs and initializes the 128-bucket map fields, zeros 0x200 bytes of bucket storage, then installs the base HashList specialization vtable. |
| `00a91360` | `TextureTrackingHashListDeletingDestructor` | ✅ | dtor | Calls the list cleanup body, frees the object only when flag bit 0 is set, and otherwise returns the original pointer. |
