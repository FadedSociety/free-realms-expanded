# Class: SoeUtil::Array<WelcomeInfo::ClaimCodeDataSource::Entry,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aceb40` | `AllocateEntryArray` | ✅ | virtual | Selects capacity from current/requested counts, writes it through param_3, reuses storage when allowed, otherwise allocates count times 0x2c bytes aligned to 4. |
| `00acebe0` | `FreeEntryArrayStorage` | ✅ | virtual | Passes the supplied pointer and alignment 4 to the memory free helper, then returns. |
| `00acec00` | `MoveOrSwapEntryArrayState` | ✅ | virtual | When param_3 is nonzero, calls param_2 vtable slot +0x0c; otherwise swaps the data pointer, count, and capacity fields between the two array objects. |
| `00acfe50` | `DestroyClaimCodeEntryArray` | ✅ | dtor | Restores the array vtable, destroys live mixed-string rows, frees aligned storage, and clears the data pointer. |
| `00acff10` | `DestroyClaimCodeArrayAndMaybeFree` | ✅ | dtor | Restores the array vtable, destroys live mixed-string rows, frees aligned storage, zeros the pointer, and frees the object when flag bit 0 is set. |
