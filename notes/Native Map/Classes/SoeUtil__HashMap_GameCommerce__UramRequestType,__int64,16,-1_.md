# Class: SoeUtil::HashMap<GameCommerce::UramRequestType,__int64,16,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d679e0` | `UramRequestTypeMap::Ctor ?` | 🟡 | ctor | Installs the typed hash-map vtable, clears entry count and 16 bucket heads, and initializes the backing storage pointer. |
| `00d69290` | `UramRequestTypeMap::Dtor ?` | 🟡 | dtor | Restores the request-type map vtable and clears all bucket entries through its map cleanup helper. |
| `00d692a0` | `UramRequestTypeMap::Allocate ?` | 🟡 | virtual | Obtains an allocation manager and requests an 0x18-byte, alignment-eight block, using a global fallback when needed. |
| `00d692e0` | `UramRequestTypeMap::Free ?` | 🟡 | virtual | Releases the supplied map allocation with alignment eight. |
| `00d69c60` | `HashMapUramRequestTypeInt64::DeletingDestructor` | ✅ | dtor | Stores the HashMap vtable, calls FUN_00d67cf0 to destroy its contents, conditionally frees this when flag bit 0 is set, and returns this otherwise. |
