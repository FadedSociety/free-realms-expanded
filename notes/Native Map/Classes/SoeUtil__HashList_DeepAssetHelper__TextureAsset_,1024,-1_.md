# Class: SoeUtil::HashList<DeepAssetHelper::TextureAsset*,1024,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a88860` | `HashListTextureAssetVSlot1Alloc20 ?` | 🟡 | virtual | Virtual slot 1 (+4) of HashList<DeepAssetHelper::TextureAsset*,1024,-1>: calls 006fb9c0, then slot +4 of the object returned by 006fb860 when non-null, else slot +4 of DAT_01bdf280 when non-null, else calls 006fb950(0x1… |
| `00a888a0` | `HashListTextureAssetVSlot2Free4` | 🟢 | virtual | Virtual slot 2 (+8) of HashList<DeepAssetHelper::TextureAsset*,1024,-1>: pushes alignment 4 and its pointer argument, calls 00401c00 (free through the memory manager), and returns with ret 4. |
| `00a8cbf0` | `HashListTextureAssetCtor` | ✅ | ctor | Constructs an empty 1024-bucket hash list of texture-asset pointers: stores the HashListMap<int,DeepAssetHelper::TextureAsset*,1024,-1> vptr, zeroes the words at +0x10, +8 and +0xc, clears 0x1000 bytes of bucket heads a… |
| `00a8ccf0` | `HashListTextureAssetDeletingDtor` | ✅ | dtor | Scalar deleting destructor of HashList<DeepAssetHelper::TextureAsset*,1024,-1>: calls 00a8b930 (HashListTextureAssetDtorFrame), then calls _free(this) through thunk 00d82463 when bit 0 of param_2 is set; returns this. |
