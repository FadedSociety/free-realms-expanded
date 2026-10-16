# Class: SoeUtil::Array<ClientAccountManager::AccountInfoGameCharacterAndFriendsDetails*,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00708e30` | `— ⚠reject` | ⚪ | virtual | Selects reuse or resize behavior from requested/current counts, updates capacity, and allocates four bytes per requested element aligned to 4. |
| `00708ed0` | `ArrayFree` | ✅ | virtual | Passes the array pointer and alignment 4 to the memory-manager free routine. |
| `00708ef0` | `ArraySwapStorage` | ✅ | virtual | When the flag is set, invokes the supplied object's virtual method; otherwise swaps data pointer and count/capacity fields. |
| `00709d40` | `ArrayDtor` | ✅ | dtor | Installs the pointer-array vtable, clears the count at +8, frees backing storage at +4 with alignment 4, nulls the storage pointer, and returns. |
| `00709ff0` | `ArrayScalarDeletingDtor` | ✅ | dtor | Stores the array vtable, clears its count, frees its pointer storage, and frees the object when the delete flag bit is set. |
