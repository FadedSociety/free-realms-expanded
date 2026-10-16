# Class: SoeUtil::List<ApiLobbyGameOption,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008e0bd0` | `ListApiLobbyGameOption_Dtor` | ✅ | dtor | Stores the ApiLobbyGameOption list vtable, then jumps to its list cleanup routine. |
| `008e0bf0` | `ListApiLobbyGameOption_Allocate` | ✅ | virtual | Runs a list allocation setup routine, then allocates an element using the thread allocator, global allocator, or fallback allocator with size 0x10 and alignment 4. |
| `008e0c30` | `FreeAligned4` | ✅ | virtual | Calls the allocator free helper with the supplied pointer and alignment 4. |
| `008e2180` | `ListApiLobbyGameOption_ScalarDeletingDtor` | ✅ | dtor | Installs the list vtable, runs list cleanup, and optionally frees the object when flag bit 0 is set. |
| `008eb9c0` | `ListDtor → SoeUtil::List<ApiLobbyGameOption,-1>::Destructor` | ✅ | dtor | Destroys embedded option list and cleans two string-like members. |
| `00a249f0` | `ApiLobbyGameOptionListInit` | ✅ | function | Installs the list vtable, zeros head, tail and count, then returns the object. |
| `00bb08d0` | `— ⚠reject` | ⚪ | function | Assignment: stores the List<ApiLobbyGameOption,-1> vtable (018037e8), clears this list's fields, and when the source is a different object drains this list through 008d4b10 and appends a copy of each source node through… |
