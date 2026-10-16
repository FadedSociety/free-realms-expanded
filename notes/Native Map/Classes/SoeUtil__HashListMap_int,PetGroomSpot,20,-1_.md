# Class: SoeUtil::HashListMap<int,PetGroomSpot,20,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ccdde0` | `PetGroomSpotMapCtor` | ✅ | ctor | Installs the typed hash-list-map vtable, zeros its size and endpoints, clears 20 bucket heads, and initializes a remaining field. |
| `00ccdfa0` | `PetGroomSpotMapAllocNode` | ✅ | method | Tries the current thread allocator, then the global allocator, and finally allocates 0x60 bytes aligned to 0x10 for a node. |
| `00ccdfe0` | `PetGroomSpotMapFreeNode` | ✅ | method | Passes the supplied node pointer to the aligned memory free routine with alignment 0x10. |
| `00cce3c0` | `PetGroomSpotMapClear` | 🟢 | method | Removes every node from the hash bucket and doubly linked list, decrements the count, and frees each node through the virtual allocator slot. |
| `00cce530` | `PetGroomSpotMapDtor` | ✅ | dtor | Restores the typed hash-list-map vtable and clears all entries through the map clear routine. |
| `00cce540` | `PetGroomSpotMapScalarDeletingDtor` | ✅ | dtor | Restores the typed hash-list-map vtable, clears the entries, and frees the allocation when the deleting flag is set. |
| `00cce570` | `PetGroomSpotMapLinkTail ?` | 🟡 | method | Links a node at the tail of the doubly linked list and at the head of its hash bucket, then increments the node count. |
| `00cce5d0` | `PetGroomSpotMapCleanup` | ✅ | dtor | Restores the typed hash-list-map vtable inside an exception cleanup frame and clears all map entries. |
| `00cce6b0` | `PetGroomSpotMapCtor` | ✅ | ctor | Initializes a typed map base with zero size and endpoints, clears the 20 buckets, and installs the HashList base vtable. |
| `00cce730` | `PetGroomSpotMapScalarDeletingDtor` | ✅ | dtor | Restores the typed map vtable, clears entries, and frees the allocation when the deleting flag is set. |
| `00cce790` | `PetGroomSpotMapInsertRecord` | 🟢 | method | Allocates and initializes a node, copies the input key into the record, links it at the list tail and into the hash bucket. |
