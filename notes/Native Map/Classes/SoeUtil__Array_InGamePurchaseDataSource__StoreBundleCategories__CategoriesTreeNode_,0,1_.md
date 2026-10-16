# Class: SoeUtil::Array<InGamePurchaseDataSource::StoreBundleCategories::CategoriesTreeNode*,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0099be40` | `— ⚠reject` | ⚪ | virtual | Virtual slot [1] of SoeUtil::Array<InGamePurchaseDataSource::StoreBundleCategories::CategoriesTreeNode*,0,1>, thiscall with three stack args (count, out-pointer, flag byte). Same policy as 0099b0f0: flag zero grows to c… |
| `0099bee0` | `ArrayCategoriesTreeNodePtrSlot2FreeBuffer` | 🟢 | virtual | Virtual slot [2] of SoeUtil::Array<InGamePurchaseDataSource::StoreBundleCategories::CategoriesTreeNode*,0,1>, one stack arg, 18 bytes. Calls 00401c00(arg,4) and returns void. |
| `0099bf00` | `ArrayCategoriesTreeNodePtrSlot3SwapOrRelease` | 🟢 | virtual | Virtual slot [3] of SoeUtil::Array<InGamePurchaseDataSource::StoreBundleCategories::CategoriesTreeNode*,0,1>, thiscall with two stack args (other array, flag byte). Flag nonzero: calls vtable slot +0xc of *param_2. Flag… |
| `0099e450` | `ArrayCategoryTreeNodeDtor` | ✅ | dtor | Restores the typed array vtable, clears its size, frees backing memory with alignment 4, and nulls the pointer. |
| `0099e480` | `ArrayCategoryTreeNodeCtor` | ✅ | ctor | Installs the typed array vtable and zeroes the data pointer, element count, and capacity fields. |
| `0099e4d0` | `ArrayCategoryTreeNodeDtorWithEH` | ✅ | dtor | Restores the array vtable, clears element count, frees the backing allocation with alignment 4, and nulls the pointer under an EH frame. |
| `0099e6e0` | `ArrayCategoryTreeNodeScalarDeletingDtor` | ✅ | virtual | Restores the typed array vtable, frees backing storage, clears pointer and size, and conditionally frees the array object. |
