# Class: SoeUtil::HashList<SoeGems::PerformanceProfiler::Snapshot::ThreadEntry,32,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00940a60` | `ThreadEntryHashListPopNode` | ✅ | virtual | Bytes add 0x194 to ECX and jump to 00940980, adjusting this before the shared free-node pop routine. |
| `00940a70` | `ThreadEntryHashListPushNode` | ✅ | virtual | Bytes add 0x194 to ECX and jump to 009409f0, adjusting this before the shared free-node push routine. |
| `009414a0` | `ThreadEntryHashListCtor` | ✅ | ctor | Constructs the embedded 32-bucket thread-entry map with 00940ac0, then installs the HashList specialization vtable and returns this. |
| `00941570` | `ThreadEntryHashListScalarDtor` | ✅ | dtor | Calls 0093ffd0 to clean the list, then conditionally frees the object and otherwise returns its pointer. |
