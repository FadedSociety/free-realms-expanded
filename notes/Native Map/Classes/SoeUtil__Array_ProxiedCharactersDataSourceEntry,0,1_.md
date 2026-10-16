# Class: SoeUtil::Array<ProxiedCharactersDataSourceEntry,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cb1f40` | `— ⚠reject` | ⚪ | virtual | Chooses a revised element capacity from the current count and request, returns existing storage when reuse conditions hold, otherwise allocates 0x38 bytes per element aligned to 8. |
| `00cb1fe0` | `ProxiedCharacterArrayFree` | ✅ | virtual | Frees the supplied buffer through the aligned memory helper using alignment 8. |
| `00cb2000` | `ProxiedCharacterArraySwapStorage` | ✅ | virtual | When the mode flag is set, dispatches a virtual operation on the supplied object; otherwise swaps the array buffer pointer and count/capacity fields with that object. |
| `00cb24c0` | `ProxiedCharacterArrayDtor` | ✅ | dtor | Restores the array vtable, destroys elements from the supplied range, frees the buffer with alignment 8, and clears its pointer. |
| `00cb24f0` | `ProxiedCharacterArrayScalarDeletingDtor` | ✅ | dtor | Restores the array vtable, destroys elements, releases aligned storage, then frees the array object when the deleting flag is set. |
