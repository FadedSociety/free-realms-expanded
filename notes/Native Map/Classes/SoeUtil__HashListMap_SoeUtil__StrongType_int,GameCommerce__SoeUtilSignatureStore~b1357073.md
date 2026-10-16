# Class: SoeUtil::HashListMap<SoeUtil::StrongType<int,GameCommerce::SoeUtilSignatureStoreBundleId,1>,AppStoreBundleDefinition,2048,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c351e0` | `AppStoreBundleMap2048Ctor` | ✅ | ctor | Installs the bundle map vtable, clears map fields, and zeroes 8192 bytes of bucket storage. |
| `00c35e80` | `AppStoreBundleMapAllocate` | ✅ | virtual | Allocates from the current or global memory manager, falling back to 0x130 bytes with alignment 8. |
| `00c35ec0` | `AppStoreBundleMapFree` | ✅ | virtual | Releases the supplied block through 00401c00 using alignment 8. |
| `00c389b0` | `AppStoreBundleMapClear` | 🟢 | dtor | Installs the base bundle-map vtable and repeatedly removes the head bundle node through 00b289d0 until empty. |
| `00c38f00` | `AppStoreBundleMapScalarDeletingDtor` | ✅ | dtor | Clears all bundle nodes through 00b289d0 and frees the map object when the low flag bit is set. |
