# Class: PlayerHousingInstanceDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cbf880` | `PlayerHousingInstanceDataSource::PlayerHousingInstanceDataSource` | ✅ | ctor | Constructs the HousingInstanceDataSource base with BaseClient.Housing.PlayerHouseList, then installs the PlayerHousingInstanceDataSource vtables. |
| `00cbf950` | `PlayerHousingInstanceDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls FUN_00cbed40 to destroy the inherited array and holder members, then frees the object when the low flag bit is set. |
