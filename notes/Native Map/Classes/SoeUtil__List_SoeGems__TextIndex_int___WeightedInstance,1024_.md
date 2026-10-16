# Class: SoeUtil::List<SoeGems::TextIndex<int>::WeightedInstance,1024>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b924b0` | `WeightedInstancePoolCtor` | ✅ | ctor | Installs base and 1024-capacity list vtables, zeros three list state fields, then initializes the embedded pool. |
| `00b92f90` | `Thunk_ThisPlus10_00b92e00` | ✅ | virtual | Adjusts ECX by 0x10 and jumps to 00b92e00. |
| `00b92fa0` | `Thunk_ThisPlus10_00b92e70` | ✅ | virtual | Adjusts ECX by 0x10 and jumps to 00b92e70. |
| `00b92ff0` | `WeightedInstancePoolDeletingDtor` | ✅ | dtor | Calls the weighted-instance pool destructor, frees this only when flag bit 0 is set, and returns this otherwise. |
