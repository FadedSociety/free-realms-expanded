# Class: SoeUtil::HashListMap<int,TinyHttp::HttpManager::HttpConnection,512,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d2efb0` | `HashListMap_HttpConnection_Ctor` | ✅ | ctor | Constructor: installs the HashListMap vptr at +0, zeroes the dwords at +4, +8, +0xC and +0x10, zeroes the 0x800-byte bucket array at this+0x14 with _memset, and returns this. |
| `00d2f570` | `AllocRouted_2298_Align8` | 🟢 | virtual | Virtual slot 1 of the HashListMap: allocates 0x2298 bytes with alignment 8 through the memory router: the thread manager from 006fb860 (vtable +4), else the global manager at DAT_01bdf280, else the fallback 006fb950. Re… |
| `00d2f5b0` | `FreeNode_Align8` | 🟢 | virtual | Virtual slot 2 of the HashListMap: frees its stack argument through 00401c00 with alignment 8 and returns (ret 4). |
| `00d32090` | `HashListMap::~HashListMap` | ✅ | dtor | Calls the connection removal routine repeatedly until the active map list is empty. |
| `00d320c0` | `HashListMap::ScalarDeletingDtor` | ✅ | dtor | Calls the HashListMap cleanup body, frees this when flag bit zero is set, and returns this otherwise. |
| `00d32450` | `HashListMap::~HashListMap → SoeUtil::HashListMap<int,TinyHttp::HttpManager::Htt…` | 🟢 | dtor | Removes all stored connections through the manager removal path, then releases the tree-backed map storage. |
| `00d32ed0` | `HashListMap::Initialize ? → SoeUtil::HashListMap<int,TinyHttp::HttpManager::Htt…` | 🟡 | init | Initializes the 512 hash buckets and four embedded object slots used by the connection pool's HashListMap storage. |
