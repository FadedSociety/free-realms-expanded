# Class: SoeUtil::Array<double,8,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d63d30` | `ArrayDouble::InlineDtor ?` | 🟡 | dtor | Frees heap storage if not using inline storage, then resets the eight-element and zero-capacity double-array state. |
| `00d63db0` | `ArrayDouble::Allocate ?` | 🟡 | virtual | Returns aligned inline storage with capacity eight when possible; otherwise delegates the request to the heap allocator. |
| `00d64080` | `ArrayDoubleInline::ScalarDeletingDtor ?` | 🟡 | dtor | Releases heap storage only when not using the inline buffer, resets base array state, and conditionally frees the object. |
| `00d64c40` | `ArrayDouble::Swap` | ✅ | virtual | Copy-constructs a temporary from param2, assigns param2 from the original this array, assigns this from the temporary, then frees the temporary storage. |
