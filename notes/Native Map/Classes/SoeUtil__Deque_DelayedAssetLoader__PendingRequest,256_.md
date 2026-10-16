# Class: SoeUtil::Deque<DelayedAssetLoader::PendingRequest,256>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ae4c00` | `PendingRequestDequeCtor` | ✅ | ctor | Installs deque vtable, initializes embedded small pointer array, and clears deque state fields. |
| `00ae4c60` | `PendingRequestDequeAlloc` | ✅ | virtual | Routes allocation through thread/global managers, then falls back to allocator with supplied size and alignment 4. |
| `00ae4cb0` | `PendingRequestDequeFree` | ✅ | virtual | Calls FreeMemory with supplied pointer and alignment 4, then returns. |
| `00ae69e0` | `SoeUtil::Deque::PendingRequestDtor` | ✅ | dtor | Installs the deque vtable, consumes pending records, releases queued objects, and destroys the embedded pointer array. |
| `00ae6a90` | `SoeUtil::Deque::PendingRequestScalarDeletingDtor` | ✅ | dtor | Calls the deque destructor, frees the object when flag bit 0 is set, and returns its pointer otherwise. |
