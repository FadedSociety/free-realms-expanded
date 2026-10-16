# Class: SoeUtil::List<T4IntegrationApi::T4Details,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ad1c60` | `AllocateT4DetailsListStorage` | ✅ | virtual | Uses the current or global memory manager to allocate 0x50 bytes aligned to 4, falling back to the local allocator when neither manager exists. |
| `00ad1ca0` | `FreeT4DetailsListStorage` | ✅ | virtual | Passes the supplied pointer and alignment 4 to the memory free helper. |
| `00ad2720` | `DestroyT4DetailsList` | ✅ | dtor | Restores the list vtable, repeatedly unlinks and destroys the head node, and returns after the list is empty. |
| `00ad2750` | `DestroyT4DetailsListAndMaybeFree` | ✅ | dtor | Restores the list vtable, removes and destroys each node, and frees the list object when flag bit 0 is set. |
