# Class: SoeUtil::HashListMap<SoeUtil::StrongType<int,GameCommerce::SoeUtilSignatureStoreBundleId,1>,SoeUtil::StrongType<int,class_GameCommerce::SoeUtilSignatureStoreBundleId,1>,1024,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b20860` | `CtorHashListMapBundleId1024` | ✅ | ctor | Constructor: stores the 1024-bucket HashListMap vtable at +0, zeroes +0x10, +8 and +0xc, memsets 0x1000 bytes at +0x14 (1024 bucket heads) to zero and zeroes +4; returns the object. |
| `00b2cb50` | `StoreBundleIdHashMap::Dtor` | ✅ | dtor | Restores the store-bundle-ID hash-map vtable and repeatedly removes each remaining entry. |
| `00b2ccd0` | `StoreBundleIdHashMap::ScalarDeletingDtor` | ✅ | virtual | Restores the hash-map vtable, removes all entries, and frees the map object if the deleting flag is set. |
