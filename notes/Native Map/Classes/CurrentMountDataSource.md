# Class: CurrentMountDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cc4190` | `CurrentMountDataSource::~CurrentMountDataSource` | ✅ | dtor | Installs the CurrentMountDataSource vtable at the object and two embedded subobjects, then destroys the GUI user-data member through 0091bf60. |
| `00cc42c0` | `CurrentMountDataSource::CurrentMountDataSource` | ✅ | ctor | Registers BaseClient.CurrentMount, installs the class vtable in three subobjects, stores param_2 at +0x254, and clears the byte at +0x258. |
| `00cc4340` | `CurrentMountDataSource::ScalarDeletingDestructor` | ✅ | dtor | Calls the CurrentMountDataSource destructor, frees this only when flag bit 0 is set, and returns this when it is retained and consumes the 4-byte delete-flag argument. |
