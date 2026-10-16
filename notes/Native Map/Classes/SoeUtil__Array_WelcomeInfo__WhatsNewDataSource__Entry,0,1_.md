# Class: SoeUtil::Array<WelcomeInfo::WhatsNewDataSource::Entry,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00acea10` | `AllocateEntryArray` | ✅ | virtual | Selects a capacity using current capacity and requested count, writes it through param_3, returns existing storage when reusable, otherwise allocates count times 0x20 bytes aligned to 4. |
| `00aceab0` | `FreeEntryArrayStorage` | ✅ | virtual | Passes the supplied pointer and alignment 4 to the memory free helper, then returns. |
| `00acead0` | `MoveOrSwapEntryArrayState` | ✅ | virtual | When param_3 is nonzero, calls param_2 vtable slot +0x0c; otherwise swaps the data pointer, count, and capacity fields between the two array objects. |
| `00acf940` | `DestroyWhatsNewEntryArray` | ✅ | dtor | Restores the array vtable, destroys its live row strings, frees storage with alignment 4, and clears the data pointer. |
| `00acfa20` | `DestroyWhatsNewArrayAndMaybeFree` | ✅ | dtor | Restores the array vtable, destroys live row strings, frees aligned storage, zeros the pointer, and frees the object when flag bit 0 is set. |
