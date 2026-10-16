# Class: BaseItemSearchEngine

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c2fa70` | `BaseItemSearchEngineDtor` | ✅ | dtor | Restores engine vtable, clears its client-profile hash map, releases a framework object, restores the token-array vtable, destroys tokens and frees token storage. |
| `00c30040` | `BaseItemSearchEngineCtor` | ✅ | ctor | Installs the engine vtable, copies eight globals, initializes the ItemSearchToken array and supporting map, resets a byte flag, and returns this. |
| `00c30130` | `BaseItemSearchEngineScalarDeletingDtor` | ✅ | dtor | Calls engine-member teardown and frees the object only when flag bit 0 is set. |
