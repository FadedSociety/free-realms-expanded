# Class: SoeUtil::HashListMap<SoeUtil::StrongType<int,GameCommerce::SoeUtilSignatureStoreBundleId,1>,SoeUtil::StrongType<int,class_GameCommerce::SoeUtilSignatureStoreBundleId,1>,128,32>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b34590` | `— ⚠reject` | ⚪ | ctor | Initializes the 128-bucket typed map, zeros counters and bucket storage, installs fixed-capacity specialization vtable, and calls a cleanup/check helper. |
| `00b34610` | `— ⚠reject` | ⚪ | virtual | Reserves 0x214 stack bytes then tail-jumps to 007e9db0; that target owns the allocator body. |
| `00b34620` | `— ⚠reject` | ⚪ | virtual | Reserves 0x214 stack bytes then tail-jumps to 007e9e20; that target owns the lookup body. |
| `00b34630` | `StoreBundleIdHashMap::Clear` | ✅ | dtor | Installs concrete map vtable, removes all chain nodes, frees extra backing node when present, resets fields, installs generic vtable and drains remaining nodes. |
| `00b34850` | `StoreBundleIdHashMap::ScalarDeletingDtor` | ✅ | dtor | Calls map destructor; when flag bit 0 is set frees object, otherwise returns original pointer. |
