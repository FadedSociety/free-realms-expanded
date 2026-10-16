# Class: SoeUtil::HashListMap<int,AssetManagement::AssetRequestInternal::AssetReadyHandler::HashEntry,1024,32>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bd2c0` | `AssetReadyHandlerHashListMap32::InitPool` | 🟢 | init | Zeroes map state, links eight inline sentinels, advances a 32-bit pointer by 0x28 elements (0xa0 bytes) per iteration, then stores capacity 0x20. |
| `0091e8a0` | `HashListMap_AssetReadyHandler_Dtor` | ✅ | dtor | Destructor of the asset-ready hash container: writes the derived HashListMap vtable, drains the node list at +8 through FUN_008fa980, frees the two bucket-array allocations held at +0x1014 (FUN_008cb030 twice and FUN_00… |
| `00920d20` | `HashListMap_AssetReadyHandler_ScalarDtor` | ✅ | dtor | Scalar deleting destructor: calls the body 0091e8a0 on this, then frees the object through _free when bit 0 of the second argument is set; returns this. |
| `00921030` | `HashListMap_AssetReadyHandler_Ctor` | ✅ | ctor | Constructor: stores the HashListMap<...,1024,-1> vtable at this, zeroes +4, +8, +0xc and +0x10, memsets the 0x1000-byte bucket array at +0x14, then stores the HashListMap<...,1024,32> vtable; calls 008bd2c0 on the sub-o… |
