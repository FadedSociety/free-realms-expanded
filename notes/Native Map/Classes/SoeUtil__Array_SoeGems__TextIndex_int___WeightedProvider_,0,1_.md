# Class: SoeUtil::Array<SoeGems::TextIndex<int>::WeightedProvider*,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b8cae0` | `WeightedProviderArray::Allocate` | ✅ | virtual | Returns current buffer when it fits, otherwise adjusts capacity, reports it, and allocates four bytes per pointer slot. |
| `00b8e8a0` | `WeightedProviderArray::ScalarDeletingDtor` | ✅ | dtor | Restores array vtable, clears size, releases external storage, and conditionally frees the array object. |
