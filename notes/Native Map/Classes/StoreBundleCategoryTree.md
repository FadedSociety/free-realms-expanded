# Class: StoreBundleCategoryTree

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c3fa70` | `StoreBundleCategoryTreeCtor` | ✅ | ctor | Installs BaseDefinition then StoreBundleCategoryTree vtables, constructs its category hash map, zeroes map arrays and counters, and clears two flag bytes. |
| `00c3fb20` | `StoreBundleCategoryTree::CanRefresh ⚠downgrade` | 🟢 | virtual | Returns true when the child pointer at +0xc is null; otherwise calls that child's virtual method and returns its boolean result. |
| `00c3fb40` | `StoreBundleCategoryTree::AppendDebugText` | ✅ | virtual | Walks category nodes through +0x13 links and appends each parent id, count, and virtual category text to the output string. |
| `00c3fc80` | `StoreBundleCategoryTreeDtor` | ✅ | dtor | Restores the category-tree vtable, clears a state byte, releases auxiliary resources, destroys the category map, and restores the BaseDefinition vtable. |
| `00c3fcf0` | `StoreBundleCategoryTree::ScalarDeletingDtor ?` | 🟡 | dtor | Calls the category-tree destructor and frees the object through the memory manager when flag bit 0 is set. |
