# Class: CoinStoreDataSource::SearchResults

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cf0950` | `CoinStoreDataSource::SearchResults::Ctor` | ✅ | ctor | Constructs the shared CoinStoreItems base and installs SearchResults vtables at three object offsets. |
| `00cf09d0` | `CoinStoreDataSource::SearchResults::Dtor` | ✅ | dtor | Restores SearchResults vtables at three offsets and invokes the shared CoinStoreItems cleanup routine. |
| `00cf12a0` | `CoinStoreDataSource::SearchResults::ScalarDeletingDtor` | ✅ | dtor | Calls SearchResults cleanup and frees the object when the deleting flag bit is set; otherwise returns its pointer. |
