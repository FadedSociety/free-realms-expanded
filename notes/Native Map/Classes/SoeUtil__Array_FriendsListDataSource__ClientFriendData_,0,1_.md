# Class: SoeUtil::Array<FriendsListDataSource::ClientFriendData*,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cb3350` | `— ⚠reject` | ⚪ | virtual | Returns current storage when capacity suffices; otherwise computes a revised capacity and allocates four bytes per pointer with alignment 4. |
| `00cb33f0` | `FriendsPointerArrayFree` | ✅ | virtual | Releases the supplied allocation through the aligned memory helper with alignment 4. |
| `00cb3410` | `FriendsPointerArraySwap` | ✅ | virtual | When the flag is set, invokes a virtual operation on the argument; otherwise swaps the array buffer pointer and size/capacity fields. |
| `00cb3850` | `FriendsPointerArrayDtor` | ✅ | dtor | Restores the array vtable, clears its count, frees its pointer buffer with alignment 4, and clears the buffer pointer. |
| `00cb3a00` | `FriendsPointerArrayScalarDeletingDtor` | ✅ | dtor | Restores the array vtable, clears its count, frees pointer storage, then frees the array object when the deleting flag is set. |
