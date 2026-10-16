# Class: SoeUtil::Array<LobbyDataSourceManager::GameConfigOptionDataSource*,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a232a0` | `— ⚠reject` | ⚪ | virtual | Capacity/allocator hook of the array class: with the force flag clear it computes a new capacity (grow to 1.25x the request when the request exceeds the current capacity, shrink to 1.2x the request when the current capa… |
| `00a23340` | `— ⚠reject` | ⚪ | virtual | Element-release hook: calls 00401c00 with the element pointer and the element size 4; nothing else. |
| `00a23360` | `SoeUtil::ArrayGCODS0SwapOrRelease` | ✅ | virtual | With the flag argument set it calls virtual slot 0xc on the other array instead of swapping; otherwise it swaps the buffer pointers (this+4 <-> other+4) and then swaps the size and capacity dwords (this+8 <-> other+8, t… |
| `00a239a0` | `— ⚠reject` | ⚪ | dtor | Destructor body of the no-inline-storage array instantiation: stores its vtable, zeroes the size field at +8, frees the buffer at +4 with element size 4 through 00401c00 and clears the buffer pointer. |
| `00a23b50` | `SoeUtil::ArrayNoInlineDeletingDtor` | ✅ | dtor | Deleting destructor of the no-inline-storage array: restores the 0,1 vtable, zeroes the size, frees the buffer with 00401c00(ptr, 4) and clears the pointer, then calls _free on the object itself when bit 0 of the flag a… |
