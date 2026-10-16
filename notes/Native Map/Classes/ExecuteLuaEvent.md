# Class: ExecuteLuaEvent

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009c51d0` | `— ⚠reject` | ⚪ | ctor | Initializes the Lua event base, its string member and parameter array, then reserves array elements based on the supplied input array size. |
| `009c5290` | `ExecuteLuaEventScalarDeletingDtor` | ✅ | dtor | Calls the event teardown routine, frees the allocation only when the deleting flag is set, and otherwise returns the original object pointer. |
