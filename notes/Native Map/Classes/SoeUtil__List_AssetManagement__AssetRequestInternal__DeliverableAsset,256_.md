# Class: SoeUtil::List<AssetManagement::AssetRequestInternal::DeliverableAsset,256>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bd370` | `DeliverableAssetList256::InitStorage` | 🟢 | init | Zeroes the list header, links 0x40 inline nodes while advancing the 32-bit pointer by 0x20 elements (0x80 bytes) per iteration, then stores capacity 0x100. |
| `0091f350` | `List_AssetDeliverable256_Ctor` | ✅ | ctor | Constructor of the DeliverableAsset list: writes the base List<...,-1> vtable, zeroes +4/+8/+0xc, switches to the List<...,256> vtable and runs FUN_008bd370 to initialize the pool/base state. |
| `0091fd40` | `ListAssetDeliverable256_AllocThunk` | ✅ | virtual | Byte-verified thunk: add ecx,0x10 then jump to ChunkPoolAlloc_2 (0091fc90), i.e. allocates one 0x20-byte chunk from the pool member at this+0x10. |
| `0091fd50` | `ListAssetDeliverable256_FreeThunk` | ✅ | virtual | Byte-verified thunk: add ecx,0x10 then jump to ChunkPoolFree_2 (0091fce0), returning one 0x20-byte chunk to the pool member at this+0x10. |
| `0091fd60` | `List_AssetDeliverable256_Dtor` | ✅ | dtor | Destructor of the DeliverableAsset list: writes the derived List<...,256> vtable, drains the node list at +4 through FUN_008faa30, frees the bucket-array header at +0x10 (FUN_008cb070 on the two arrays plus FUN_00401c00… |
| `00920d40` | `List_AssetDeliverable256_ScalarDtor` | ✅ | dtor | Scalar deleting destructor: calls the body 0091fd60 on this, then frees the object through _free when bit 0 of the second argument is set; returns this. |
