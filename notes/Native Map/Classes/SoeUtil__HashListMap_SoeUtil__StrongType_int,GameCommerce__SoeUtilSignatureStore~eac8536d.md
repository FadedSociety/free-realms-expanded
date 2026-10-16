# Class: SoeUtil::HashListMap<SoeUtil::StrongType<int,GameCommerce::SoeUtilSignatureStoreBundleId,1>,SoeUtil::StrongType<int,class_GameCommerce::SoeUtilSignatureStoreBundleId,1>,1024,256>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b30790` | `SignatureStoreBundleIdHashMap::Ctor` | ✅ | ctor | Installs the 1024-bucket hash-list vtable, zeroes list pointers and count, clears 0x1000 bytes of bucket heads, and initializes a subobject. |
| `00b30810` | `SignatureStoreBundleIdHashMap::Dtor` | ✅ | dtor | Unlinks every hashed node, frees the backing tree/allocation, clears list endpoints, and restores the generic map vtable. |
| `00b308f0` | `AdjustThisAndJump_00b308f0` | ✅ | virtual | Adds 0x1014 to ECX, then jumps to 0x008f37a0; this adjustor thunk does not contain the target body. |
| `00b30900` | `AdjustThisAndJump_00b30900` | ✅ | virtual | Adds 0x1014 to ECX, then jumps to 0x008f3810; this adjustor thunk has no separate target body. |
| `00b30930` | `SignatureStoreBundleIdHashMap::ScalarDeletingDtor` | ✅ | dtor | Calls the specialized map destructor, frees the allocation when flag bit 0 is set, and otherwise returns the map pointer. |
