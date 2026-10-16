# Class: SoeUtil::List<GameAccountManagement::NameSuggestion,5>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00955550` | `NameSuggestionListCtor ?` | 🟡 | ctor | Installs the fixed-capacity NameSuggestion list vtable, clears counters and flags, initializes four inline links, and sets capacity to five. |
| `00955fc0` | `NameSuggestionList::Pop ?` | 🟡 | virtual | Adds 0x10 to the object pointer and jumps to 00955ee0; this is an adjustor thunk, not the callee body. |
| `00955fd0` | `NameSuggestionList::Push ?` | 🟡 | virtual | Adds 0x10 to the object pointer and jumps to 00955f50; this is an adjustor thunk, not the callee body. |
| `00955fe0` | `NameSuggestionListDtor ?` | 🟡 | dtor | Installs fixed and dynamic list vtables while clearing linked elements, counters and flag; destroys its external storage and restores the base list vtable. |
| `009560b0` | `NameSuggestionList::ScalarDeletingDtor ?` | 🟡 | dtor | Calls the list destructor and frees the object when deletion flag bit 0 is set; otherwise returns the pointer. |
