# Class: SoeUtil::HashListMap<int,SoeGems::PerformanceProfiler::Snapshot::ThreadEntry,32,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0093ffd0` | `DestroyThreadEntryHashMap` | ✅ | dtor | Installs map vtables, drains hash nodes twice around freeing the pooled red-black tree, resets pool metadata, and returns with the base map vtable restored. |
| `00940aa0` | `ThreadEntryHashMapScalarDtor` | ✅ | dtor | Calls 0093ffd0 to drain the map, then conditionally frees the object based on the deleting flag. |
| `00940ac0` | `ThreadEntryHashMapCtor` | ✅ | ctor | Initializes a 32-bucket map, resets free-list pointers and counters, aligns an embedded sentinel, and sets the pool mode to 1. |
