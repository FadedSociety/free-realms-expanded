# Class: SoeUtil::HashListMap<SoeUtil::StrongType<int,GameCommerce::SoeUtilSignatureStoreBundleId,1>,SoeUtil::StrongType<int,class_GameCommerce::SoeUtilSignatureStoreBundleId,1>,128,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b209a0` | `CtorHashListMapBundleId128` | ✅ | ctor | Constructor: stores the 128-bucket bundle-id HashListMap vtable at +0, zeroes +0x10, +8 and +0xc, memsets 0x200 bytes at +0x14 (128 bucket heads) to zero and zeroes +4; returns the object. |
| `00b30c70` | `StoreBundleIdMap128::ClearNodes` | ✅ | dtor | Writes this map vtable and repeatedly removes the current node through FUN_00b29020 until no nodes remain. |
| `00b31320` | `StoreBundleIdMap128::ScalarDeletingDtor` | ✅ | dtor | Removes nodes through FUN_00b29020, then frees the map allocation when flag bit 0 is set or returns the pointer otherwise. |
