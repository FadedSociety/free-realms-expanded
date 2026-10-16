# Class: MysteryChestDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009c6df0` | `MysteryChestDataSourceUpdateField` | ✅ | method | When the datasource is not marked closed, invokes a virtual update on its owner, then calls a shared cleanup helper. |
| `009c6e10` | `MysteryChestDataSourceFieldName` | ✅ | virtual | Maps field indices 0 through 10 to chest identifier, display strings and images, remaining seconds, membership, random activity, image set, and allowed-state names. |
| `009c8160` | `RefreshMysteryChestDataSource` | ✅ | virtual | Clears the temporary pointer list, collects current chest records, sorts that list with a local comparator, emits each record to the datasource, and sets the refreshed flag. |
| `009c8ab0` | `— ⚠reject` | ⚪ | virtual | Looks up a datasource row, returns the selected chest field for indices 0 through 10, computes remaining time, and resolves random activity/image-set fields through related maps. |
| `009c8f70` | `MysteryChestDataSource::Dtor` | ✅ | dtor | Restores the data-source vtables at the object base and two subobjects, then destroys its GUI user-data table member. |
| `009c9250` | `MysteryChestDataSource::Ctor` | ✅ | ctor | Initializes the embedded base, installs data-source vtables, stores supplied values at +0x254/+0x258, and clears byte +0x25c. |
| `009c92e0` | `MysteryChestDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls the data-source destructor, frees the object when flag bit 0 is set, and otherwise returns its pointer. |
