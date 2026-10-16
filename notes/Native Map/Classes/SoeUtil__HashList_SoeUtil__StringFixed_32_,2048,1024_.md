# Class: SoeUtil::HashList<SoeUtil::StringFixed<32>,2048,1024>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d08b40` | `— ⚠reject` | ⚪ | virtual | Forwards arguments to pool allocation helper and returns its result. |
| `00d08b50` | `HashList::FreeNode → HashList::FreeNode_ThisPlus2014Thunk` | 🟢 | virtual | Forwards the supplied node to the pool free-list helper. |
| `00d09320` | `StringHashList_Init` | ✅ | ctor | Zeros map counters and buckets, initializes the base map, then installs StringFixed<32> HashList vtable. |
| `00d09360` | `StringHashList::ScalarDeletingDtor` | ✅ | virtual | Runs map cleanup, frees object if requested, and otherwise returns the original pointer. |
