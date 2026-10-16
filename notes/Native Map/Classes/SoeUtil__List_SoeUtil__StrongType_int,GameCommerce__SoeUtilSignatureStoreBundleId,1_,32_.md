# Class: SoeUtil::List<SoeUtil::StrongType<int,GameCommerce::SoeUtilSignatureStoreBundleId,1>,32>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008f2ca0` | `ListStoreBundleId_Ctor` | ✅ | ctor | Installs the dynamic list vtable, clears its three header fields, switches to the inline-32 list vtable, and initializes the embedded sentinel pool. |
| `008f2d20` | `ListStoreBundleId_Dtor` | ✅ | dtor | Installs the inline-list vtable, removes list nodes, recursively frees pooled tree nodes and the block, clears pool fields, then installs the dynamic list vtable and removes its nodes. |
| `008f2dd0` | `— ⚠reject` | ⚪ | virtual | Adds 0x14 to ECX and jumps to 008f2b00; the eight-byte body is an adjustor thunk. |
| `008f2de0` | `— ⚠reject` | ⚪ | virtual | Adds 0x14 to ECX and jumps to 008f2b70; the eight-byte body is an adjustor thunk. |
