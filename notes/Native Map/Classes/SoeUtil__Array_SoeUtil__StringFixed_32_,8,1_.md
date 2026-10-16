# Class: SoeUtil::Array<SoeUtil::StringFixed<32>,8,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d63e60` | `ArrayString32::InlineDtor ?` | 🟡 | dtor | Releases string elements and heap storage when applicable, then resets both inline-capacity and zero-capacity array state. |
| `00d63f30` | `ArrayString32::Allocate ?` | 🟡 | virtual | Returns aligned inline storage with capacity eight when possible; otherwise delegates the request to the heap allocator. |
| `00d64120` | `ArrayString32Inline::ScalarDeletingDtor ?` | 🟡 | dtor | Calls the inline array destructor and conditionally frees the containing object when the deleting flag is set. |
| `00d66010` | `ArrayString32::Reserve ? → SoeUtil::Array<StringFixed<32>,8,1>::Reallocate(newC…` | 🟡 | function | Requests capacity through the array's allocator, destroys old string elements, releases old storage, and stores the replacement pointer and capacity. |
| `00d661f0` | `ArrayString32::AppendRange ?` | 🟡 | function | Ensures capacity for the appended count, constructs each StringFixed<32> element at the destination tail, advances count, and returns. |
| `00d662a0` | `ArrayString32::GetOrCreateAt ? → SoeUtil::Array<StringFixed<32>,8,1>::PrepareSl…` | 🟡 | function | Returns an existing element after clearing it, or grows the array to index+1, initializes intervening strings to the shared empty value, and returns the indexed slot. |
| `00d66420` | `ArrayString32::EnsureCapacity ? → GameCommerce::MarketingDataElementInstance<St…` | 🟡 | virtual | Checks requested count against capacity and asks the StringFixed<32> array allocator to grow when needed. |
| `00d66790` | `ArrayString32::Swap` | ✅ | virtual | Appends param2 elements to a temporary and truncates param2; appends param1 elements to param2 and truncates param1; then appends the temporary elements to param1, truncates the temporary, and frees its storage. |
