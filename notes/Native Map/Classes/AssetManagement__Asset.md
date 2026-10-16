# Class: AssetManagement::Asset

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0071bb80` | `AssetConstructor` | ✅ | ctor | Constructs RefCountedImplicit base, installs Asset vtable, initializes embedded IString to empty, and clears asset fields. |
| `0071bc20` | `AssetDestructor` | ✅ | dtor | Restores IString and RefCounted base vtables, releases owned string storage and resets embedded reference-count fields. |
| `0071bcb0` | `AssetScalarDeletingDtor` | ✅ | dtor | Calls Asset destructor and frees object when deleting flag bit is set. |
| `0071c0c0` | `Asset::SharedVirtualMethod2` | ✅ | virtual | Creates temporary IString, calls 006f9e60 for temporary and owner, releases any owned buffer, and returns. |
