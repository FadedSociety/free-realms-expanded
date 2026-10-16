# Class: PlayerTitleDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cc58c0` | `PlayerTitleDataSource::~PlayerTitleDataSource` | ✅ | dtor | Restores the PlayerTitleDataSource vtable in the object and two embedded subobjects, then destroys the GUI user-data member through 0091bf60. |
| `00cc5930` | `PlayerTitleDataSource::PlayerTitleDataSource` | ✅ | ctor | Registers BaseClient.PlayerTitles, installs the class vtable in three subobjects, stores constructor values at +0x254 and +0x258, and clears byte +0x25c. |
| `00cc59c0` | `PlayerTitleDataSource::ScalarDeletingDestructor` | ✅ | dtor | Calls the PlayerTitleDataSource destructor, frees this only when flag bit 0 is set, otherwise returns this, and consumes the 4-byte delete-flag argument. |
