# Class: FeaturedDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cf9dc0` | `FeaturedDataSource::GetColumnName` | ✅ | virtual | Returns names for System, Guid, OwnerName, Name, OwnerGuid, Snapshot, Rating, and NumVotes columns. |
| `00cf9f40` | `FeaturedDataSource::GetRowCount ?` | 🟡 | virtual | If the source is dirty, refreshes its rows, then returns the collection count at offset 8. |
| `00cfa100` | `FeaturedDataSource::GetCell` | ✅ | virtual | Resolves a row and returns system, GUID, owner, name, snapshot, rating, or vote count fields; missing data yields an error string. |
| `00cfa330` | `FeaturedDataSource::Cleanup ?` | 🟡 | dtor | Restores the FeaturedDataSource vtables and destroys its embedded GUI userdata table. |
| `00cfa460` | `FeaturedDataSource::FeaturedDataSource` | ✅ | ctor | Initializes the datasource base using BaseClient.FeaturedRating, installs the class vtables, stores two arguments, and clears its populated flag. |
| `00cfa4f0` | `FeaturedDataSource::ScalarDeletingDtor ?` | 🟡 | dtor | Calls the class cleanup and frees the object when the delete flag's low bit is set. |
