# Class: SoeUtil::HashList<AssetManagement::AssetRequestInternal::AssetReadyHandler::HashEntry,1024,32>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0091f330` | `HashListAssetReady_AllocThunk` | ✅ | virtual | Byte-verified thunk: add ecx,0x1014 then jump to ChunkPoolAlloc (0091f250), i.e. allocates one 0x28-byte chunk from the pool member that starts at this+0x1014. |
| `00924b00` | `HashList_AssetReadyHandler_Ctor` | ✅ | ctor | Constructor of the HashList base: calls 00921030 on this (the HashListMap<...,1024,-1>/32 constructor, bytes: call with ECX = this), then stores the HashList<...,1024,32> vtable at this and returns this. |
| `009251e0` | `HashList_AssetReadyHandler_ScalarDtor` | ✅ | dtor | Scalar deleting destructor: calls the body 0091e8a0 on this (bytes: call after the SEH frame setup), then frees the object through _free when bit 0 of the second argument is set; returns this. |
