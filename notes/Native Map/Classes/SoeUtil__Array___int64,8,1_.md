# Class: SoeUtil::Array<__int64,8,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d63c00` | `ArrayInt64::InlineDtor ?` | 🟡 | dtor | Frees heap storage when the array is not using its inline buffer, then resets both inline and zero-capacity array state. |
| `00d63c80` | `ArrayInt64::Allocate ?` | 🟡 | virtual | For requests above eight elements invokes the heap allocator; otherwise reports capacity eight and returns the aligned inline buffer. |
| `00d63fe0` | `ArrayInt64Inline::ScalarDeletingDtor ?` | 🟡 | dtor | Releases heap storage only when not using the inline buffer, resets the base array state, and conditionally frees the object. |
| `00d64ba0` | `ArrayInt64::Swap` | ✅ | virtual | Copy-constructs a temporary from param2, assigns param2 from the original this array, assigns this from the temporary, then frees the temporary storage. |
