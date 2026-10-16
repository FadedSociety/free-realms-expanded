# Class: SoeUtil::HashListMap<int,ProfileItem,4,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ae1b40` | `ProfileItemHashAlloc` | ✅ | virtual | Initializes memory management, asks the thread/global manager for an allocation, and falls back to FUN_006fb950(0x18,4). |
| `00ae1b80` | `ProfileItemHashFree` | ✅ | virtual | Pushes the supplied pointer and alignment 4 to FreeMemory (00401c00), then returns. |
| `00ae1ca0` | `ProfileItemHashList::ScalarDeletingDtor → SoeUtil::HashListMap<int,ProfileItem,…` | ✅ | dtor | Installs the map vtable, removes nodes until head is null, returns this without delete flag bit 0, otherwise frees this. |
| `00ae1ce0` | `ProfileItemHashMapDtor → ProfileItemHashListMapDtor` | ✅ | dtor | Under SEH, installs the map vtable and removes current head nodes with FUN_008bfd60 until none remain. |
| `00ae1e30` | `ProfileItemHashMapInit` | ✅ | ctor | Installs map then list vtables, clears count/head/tail and the four bucket heads, and returns. |
