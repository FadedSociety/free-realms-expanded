# Class: HouseRatingDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cf9d40` | `HouseRatingDataSource::GetColumnName` | ✅ | virtual | Returns names for HouseGuid, OwnerName, HouseName, OwnerGuid, Snapshot, Rating, and NumVotes columns. |
| `00cf9f10` | `HouseRatingDataSource::GetRowCount ?` | 🟡 | virtual | If the source is dirty, refreshes its rows, then returns the collection count at offset 8. |
| `00cf9f70` | `HouseRatingDataSource::GetCell` | ✅ | virtual | Resolves a row and returns house GUID, owner name, house name, owner GUID, snapshot, rating, or vote count; missing data yields an error string. |
| `00cfa2b0` | `HouseRatingDataSource::Cleanup ?` | 🟡 | dtor | Restores the HouseRatingDataSource vtables and destroys its embedded GUI userdata table. |
| `00cfa3b0` | `HouseRatingDataSource::HouseRatingDataSource` | ✅ | ctor | Initializes the datasource base using BaseClient.HouseRating, installs the class vtables, stores two arguments, and clears its populated flag. |
| `00cfa440` | `HouseRatingDataSource::ScalarDeletingDtor ?` | 🟡 | dtor | Calls the class cleanup and frees the object when the delete flag's low bit is set. |
