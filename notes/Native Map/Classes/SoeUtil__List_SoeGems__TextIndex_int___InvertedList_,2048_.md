# Class: SoeUtil::List<SoeGems::TextIndex<int>::InvertedList*,2048>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b90f00` | `InvertedListPoolCtor` | ✅ | ctor | Installs base and 2048-capacity list vtables, zeros three list state fields, then initializes the embedded node pool. |
| `00b919d0` | `Thunk_ThisPlus10_00b91850` | ✅ | virtual | Adjusts ECX by 0x10 and jumps to 00b91850. |
| `00b919e0` | `Thunk_ThisPlus10_00b918c0` | ✅ | virtual | Adjusts ECX by 0x10 and jumps to 00b918c0. |
| `00b92fb0` | `InvertedListPoolDeletingDtor` | ✅ | dtor | Calls the inverted-list pool destructor, frees this only when flag bit 0 is set, and returns this otherwise. |
