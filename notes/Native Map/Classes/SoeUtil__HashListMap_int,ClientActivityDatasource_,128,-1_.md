# Class: SoeUtil::HashListMap<int,ClientActivityDatasource*,128,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ce8e90` | `ActivityDatasourceMapCtor ?` | 🟡 | ctor | Installs the typed hash-list map vtable, clears counters and list endpoints, zeros all 128 bucket heads, and clears the base pointer. |
| `00ce91b0` | `SoeUtil::HashListMap::AllocateNode ?` | 🟡 | method | Initializes the container allocator, asks the current thread manager for memory, falls back to the global manager, then allocates 0x14 bytes aligned to 4. |
| `00ce91f0` | `SoeUtil::HashListMap::FreeNode ?` | 🟡 | method | Frees the supplied node pointer through the memory manager with alignment 4. |
| `00ce9320` | `SoeUtil::HashListMap::RemoveNode ?` | 🟡 | method | Finds node in bucket selected by key low 7 bits, unlinks it from the bucket chain and doubly linked list, decrements count, and calls the virtual node free slot. |
| `00ce96d0` | `SoeUtil::HashListMap::Clear ?` | 🟡 | method | Walks linked nodes, obtains and releases each held object handle, unlinks each node, decrements object references, and deletes objects reaching zero. |
| `00ce9730` | `SoeUtil::HashListMap::RemoveByPredicate ? ⚠rename` | 🟡 | method | Hashes the key, scans matching bucket nodes, invokes a node method and predicate helper, then releases/unlinks/decrements references for matches. |
| `00ce9a10` | `SoeUtil::HashListMap::RemoveHeadUntilEmpty ?` | 🟡 | method | Repeatedly removes the node at list head until the head pointer is null. |
| `00ce9b20` | `SoeUtil::HashListMap::DtorBody ?` | 🟡 | dtor | Restores the typed map vtable and repeatedly unlinks the current head node until the map is empty. |
| `00ce9b80` | `SoeUtil::HashListMap::ScalarDeletingDtor ? → SoeUtil::HashListMap<int,ClientAct…` | 🟡 | dtor | Restores map vtable, removes all nodes through the head-removal helper, and conditionally frees the object. |
| `00ce9bc0` | `SoeUtil::HashListMap::ClearForUnwind ?` | 🟡 | dtor | Restores map vtable, repeatedly removes its head node until empty, and restores the exception frame. |
| `00ce9ce0` | `SoeUtil::HashListMap::ActivityListCtor ?` | 🟡 | ctor | Initializes map counters, endpoints, bucket array, then switches the primary vtable to the HashList base implementation. |
| `00cea260` | `SoeUtil::HashListMap::InsertNode ?` | 🟡 | method | Allocates a node, stores its payload/key, links it at list tail and bucket head selected by key low 7 bits, then increments count. |
