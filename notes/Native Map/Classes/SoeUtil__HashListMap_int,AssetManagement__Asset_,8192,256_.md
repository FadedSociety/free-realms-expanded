# Class: SoeUtil::HashListMap<int,AssetManagement::Asset*,8192,256>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008f2df0` | `HashListMapAsset_Dtor` | ✅ | dtor | Installs the Asset map vtable, unlinks every head node, recursively frees the chunk-index tree and block, clears pool fields, then installs the dynamic base vtable and drains remaining nodes. |
| `008f3880` | `HashListMapAsset_Slot1Adjustor` | ✅ | virtual | Adds 0x8014 to ECX and jumps to 008f37a0; this is an adjustor thunk, not the target function body. |
| `008f3890` | `HashListMapAsset_Slot2Adjustor` | ✅ | virtual | Adds 0x8014 to ECX and jumps to 008f3810; this is an adjustor thunk, not the target function body. |
