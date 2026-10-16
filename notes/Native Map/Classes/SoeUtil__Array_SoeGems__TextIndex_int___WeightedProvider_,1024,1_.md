# Class: SoeUtil::Array<SoeGems::TextIndex<int>::WeightedProvider*,1024,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b8cc10` | `WeightedProviderInlineArray::FreeExternal` | ✅ | virtual | Calls the aligned free helper only when the supplied pointer differs from the inline-buffer address. |
| `00b8e8e0` | `WeightedProviderInlineArray::ScalarDeletingDtor` | ✅ | dtor | Calls the 1024-inline array destructor and conditionally frees the array object. |
| `00b8e970` | `— ⚠reject` | ⚪ | ctor | Initializes array pointer, size, capacity and inline storage, then installs the 1024-inline specialization vtable. |
