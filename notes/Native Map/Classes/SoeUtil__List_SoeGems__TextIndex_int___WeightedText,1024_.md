# Class: SoeUtil::List<SoeGems::TextIndex<int>::WeightedText,1024>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b919f0` | `WeightedTextPoolCtor` | ✅ | ctor | Installs base and 1024-capacity list vtables, zeros three list state fields, then initializes the embedded pool. |
| `00b92490` | `Thunk_ThisPlus10_00b92330` | ✅ | virtual | Adjusts ECX by 0x10 and jumps to 00b92330. |
| `00b924a0` | `Thunk_ThisPlus10_00b92380` | ✅ | virtual | Adjusts ECX by 0x10 and jumps to 00b92380. |
| `00b92fd0` | `WeightedTextPoolDeletingDtor` | ✅ | dtor | Calls the weighted-text pool destructor, frees this only when flag bit 0 is set, and returns this otherwise. |
