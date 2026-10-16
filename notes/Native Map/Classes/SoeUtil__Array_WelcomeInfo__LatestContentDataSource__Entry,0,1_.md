# Class: SoeUtil::Array<WelcomeInfo::LatestContentDataSource::Entry,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00acec70` | `AllocateEntryArray` | ✅ | virtual | Selects capacity from current/requested counts, writes it through param_3, reuses storage when allowed, otherwise allocates count times 8 bytes aligned to 4. |
| `00aced10` | `FreeEntryArrayStorage` | ✅ | virtual | Passes the supplied pointer and alignment 4 to the memory free helper, then returns. |
| `00aced30` | `MoveOrSwapEntryArrayState` | ✅ | virtual | When param_3 is nonzero, calls param_2 vtable slot +0x0c; otherwise swaps the data pointer, count, and capacity fields between the two array objects. |
| `00acee50` | `DestroyEntryArray` | ✅ | dtor | Restores the array vtable, clears its count, frees its storage with alignment 4, and zeros the data pointer. |
| `00acef10` | `DestroyEntryArrayAndMaybeFree` | ✅ | dtor | Restores the array vtable, clears count, frees aligned storage, zeros the pointer, and frees the array object when flag bit 0 is set. |
