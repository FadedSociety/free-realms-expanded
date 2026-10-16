# Class: SoeUtil::Set<SoeUtil::StrongType<int,GameCommerce::SoeUtilSignatureStoreBundleId,1>,32,0>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009b4060` | `— ⚠reject` | ⚪ | ctor | Initializes set storage and installs the fixed-capacity-32 StrongType store-bundle ID set vtable, then calls FUN_007e8680 for remaining setup. |
| `009b40e0` | `StoreBundleIdSetDtor` | ✅ | dtor | Recursively frees tree nodes and backing storage, resets set fields, and restores the dynamic and generic StrongType set vtables during teardown. |
| `009b41e0` | `SetAdjustorThunk ? → StoreBundleIdSet::AcquireFreeNodeAdjustorThunk` | 🟡 | function | Adds 0x0c to ECX, then jumps to 007e9db0; bytes show an adjustor thunk, not the destination body. |
| `009b4260` | `StoreBundleIdSet::ScalarDeletingDtor` | ✅ | virtual | Calls set cleanup FUN_009b40e0 and frees the object when deletion flag bit 0 is set. |
| `009b5060` | `StoreBundleIdSetCopyCtor ?` | 🟡 | ctor | Initializes the StrongType store-bundle set, then copies contents from the supplied source through FUN_009b1350. |
