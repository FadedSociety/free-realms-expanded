# Class: SoeUtil::HashListMap<int,SoeUtil::RefPointer<LoadingScreenData>,16,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ca82c0` | `HashListMap_LoadingScreenData::CtorBase` | ✅ | ctor | Constructs the base HashListMap: installs the HashListMap vtable at +0, zeroes the words at +0x4, +0x8 and +0xc, clears the 0x40-byte bucket array at +0x14 with _memset, and zeroes +0x4 counter words. |
| `00ca88c0` | `HashListMap_LoadingScreenData::AllocNode20` | ✅ | virtual | Virtual slot 1 of the hash map (also the HashList slot 1): allocates a 0x14-byte node through the current thread's memory manager (006fb860, slot 1), else the global manager DAT_01bdf280, else the CRT fallback 006fb950(… |
| `00ca8900` | `HashListMap_LoadingScreenData::FreeNode` | ✅ | virtual | Virtual slot 2 of the hash map: frees the node argument through FreeMemory with alignment 4 (00401c00) and returns, popping 4 bytes. |
| `00caabe0` | `HashListMap_LoadingScreenData::DtorBody` | ✅ | dtor | Destructor body of the LoadingScreenData id map: stores the map vtable, then repeatedly removes the head node through 00ca9530 until none is left. |
| `00caac10` | `— ⚠reject` | ⚪ | dtor | Scalar deleting destructor of the LoadingScreenData id map (vtable slot 0): stores the vtable, removes every node through 00ca9530, and frees this through _free when bit 0 of the flag argument is set. |
| `00caac50` | `HashListMap_LoadingScreenData::DtorBodyB` | ✅ | dtor | EH-framed variant of the id-map destructor body: stores the map vtable, removes every node through 00ca9530 and returns. |
