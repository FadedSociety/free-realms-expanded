# Class: MountListDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cc4110` | `MountListDataSource::~MountListDataSource` | ✅ | dtor | Installs the MountListDataSource vtable at the object and two embedded subobjects, destroys the map member through 00b058a0, then destroys the GUI user-data member through 0091bf60. |
| `00cc4200` | `MountListDataSource::MountListDataSource` | ✅ | ctor | Registers BaseClient.MountList, stores the two constructor arguments at +0x254 and +0x258, installs the class vtable in three subobjects, constructs the mount-info map, and clears flags at +0x2b0 and +0x2b1. |
| `00cc42a0` | `MountListDataSource::ScalarDeletingDestructor` | ✅ | dtor | Calls the MountListDataSource destructor, frees this only when flag bit 0 is set, and returns this when it is retained and consumes the 4-byte delete-flag argument. |
