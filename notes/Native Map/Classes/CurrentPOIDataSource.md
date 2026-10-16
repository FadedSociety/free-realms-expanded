# Class: CurrentPOIDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c8e350` | `CurrentPOIDataSource::ColumnName` | ✅ | virtual | Maps indices 0, 1, and 2 to POIName, POIId, and AreaId; other indices return an error label. |
| `00c8e3c0` | `CurrentPOIDataSource::GetColumnValue` | ✅ | virtual | For column 0 reads the current POI name; columns 1 and 2 format the POI ID and area ID into the output value. |
| `00c8e440` | `CurrentPOIDataSource::SetCurrentPOI` | 🟢 | method | If either POI or area ID changed, updates the name string and two IDs, then notifies virtual slot 0x10. |
| `00c8e490` | `CurrentPOIDataSource::Dtor` | ✅ | dtor | Restores source vtables, releases its narrow string buffer, and destroys embedded GUI data source state. |
| `00c8e550` | `CurrentPOIDataSource::CurrentPOIDataSource` | ✅ | ctor | Registers BaseClient.CurrentPOI, installs source and string vtables, initializes string storage, and clears current POI and area IDs. |
| `00c8e5f0` | `CurrentPOIDataSource::ScalarDeletingDtor` | 🟢 | dtor | Calls CurrentPOIDataSource cleanup and frees the object when deleting flag bit 0 is set. |
