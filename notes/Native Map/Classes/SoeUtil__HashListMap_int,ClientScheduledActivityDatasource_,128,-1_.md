# Class: SoeUtil::HashListMap<int,ClientScheduledActivityDatasource*,128,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ce8f00` | `ScheduledDatasourceMapCtor ?` | 🟡 | ctor | Installs the scheduled-activity hash-list map vtable, clears counters/list endpoints, zeros all 128 bucket heads, and clears the base pointer. |
| `00ce9210` | `SoeUtil::HashListMap::AllocateScheduledNode ?` | 🟡 | method | Initializes the container allocator, asks the current thread manager for memory, falls back to the global manager, then allocates 0x14 bytes aligned to 4. |
| `00ce9250` | `SoeUtil::HashListMap::FreeScheduledNode ?` | 🟡 | method | Frees the supplied node pointer through the memory manager with alignment 4. |
| `00ce93d0` | `SoeUtil::HashListMap::RemoveScheduledNode ?` | 🟡 | method | Finds node in bucket selected by key low 7 bits, unlinks it from bucket and doubly linked list, decrements count, and calls virtual node free slot. |
| `00ce9870` | `SoeUtil::HashListMap::ClearScheduled ?` | 🟡 | method | Walks scheduled nodes, obtains/releases object handles, unlinks nodes, decrements object references, and deletes objects reaching zero. |
| `00ce98d0` | `SoeUtil::HashListMap::RemoveScheduledByPredicate ? ⚠rename` | 🟡 | method | Hashes the key, scans matching scheduled bucket nodes, evaluates their predicate, and releases/unlinks/decrements references for matching objects. |
| `00ce9a30` | `SoeUtil::HashListMap::RemoveScheduledHeadUntilEmpty ?` | 🟡 | method | Repeatedly removes the node at list head until the head pointer is null. |
| `00ce9b50` | `SoeUtil::HashListMap::ScheduledDtorBody ?` | 🟡 | dtor | Restores scheduled map vtable and repeatedly unlinks the current head node until the map is empty. |
| `00ce9c30` | `SoeUtil::HashListMap::ScheduledScalarDeletingDtor ? → SoeUtil::HashListMap<int,…` | 🟡 | dtor | Restores scheduled map vtable, removes all nodes through the head-removal helper, and conditionally frees the object. |
| `00ce9c70` | `SoeUtil::HashListMap::ScheduledClearForUnwind ?` | 🟡 | dtor | Restores scheduled map vtable, repeatedly removes its head node until empty, and restores exception frame. |
| `00ce9d60` | `SoeUtil::HashListMap::ScheduledListCtor ?` | 🟡 | ctor | Initializes scheduled map counters, endpoints, and bucket array, then installs the HashList base vtable. |
| `00cea300` | `SoeUtil::HashListMap::InsertScheduledNode ?` | 🟡 | method | Allocates a node, stores its payload/key, links it at list tail and bucket head selected by key low 7 bits, then increments count. |
