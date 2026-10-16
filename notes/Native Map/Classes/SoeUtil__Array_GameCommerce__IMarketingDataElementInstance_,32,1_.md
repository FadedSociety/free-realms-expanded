# Class: SoeUtil::Array<GameCommerce::IMarketingDataElementInstance*,32,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d63ad0` | `ArrayMarketingElementPointers::InlineDtor ?` | 🟡 | dtor | Frees the pointer buffer only when it is not the inline buffer, resets the count, installs the zero-capacity vtable, and clears the pointer. |
| `00d63b50` | `ArrayMarketingElementPointers::Allocate ?` | 🟡 | virtual | For requests above 32 elements calls the out-of-line allocator; otherwise reports capacity 32 and returns the aligned inline buffer. |
| `00d64a10` | `ArrayMarketingElementPointers::InlineDtor ?` | 🟡 | dtor | Frees the pointer buffer when it is not inline storage, clears count and pointer, installs the zero-capacity array vtable, and resets base state. |
| `00d64aa0` | `ArrayMarketingElementPointers::ScalarDeletingDtor ?` | 🟡 | dtor | Releases heap storage if not inline, resets array state and vtable, then frees the object when the deleting flag is set. |
| `00d64b00` | `ArrayMarketingElementPointers::Swap` | ✅ | virtual | Stages the second pointer array in a temporary, copies the first array into the second, copies the temporary into the first, clears the temporary, and frees its storage. |
| `00d64d70` | `ArrayMarketingElementPointers::InlineDtor ?` | 🟡 | dtor | Releases the heap pointer buffer if not inline, clears count and buffer, then restores the zero-capacity array vtable and state. |
| `00d65810` | `ArrayMarketingElementPointers::ScalarDeletingDtor ? → GameCommerce::MarketingDa…` | 🟡 | dtor | Releases heap pointer storage if not inline, resets count and array state, then conditionally frees the object. |
