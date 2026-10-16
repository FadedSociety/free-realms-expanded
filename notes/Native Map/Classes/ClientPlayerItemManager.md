# Class: ClientPlayerItemManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009912f0` | `ClientPlayerItemManager::RefreshDerivedState` | ✅ | method | Notifies the manager listener, refreshes boost-currency and name-change derived values from item totals, then checks the Factory.Tools data source and updates it when available. |
| `00991430` | `ClientPlayerItemManager::Refresh` | ✅ | virtual | Calls the client item manager's derived-state refresh routine and returns. |
| `00991440` | `— ⚠reject` | ⚪ | virtual | Calls the derived-state refresh, then sets character, pet, and house name-change properties to true when the changed key matches each corresponding item key. |
| `009918d0` | `ClientPlayerItemManager::Ctor` | ✅ | ctor | Initializes the generic item manager, installs the client manager vtable, creates named inventory/search/storage data sources and lists for equipment and vehicles, and registers rental, expired-rental, and vehicle data … |
| `009921a0` | `ClientPlayerItemManager::Refresh` | ✅ | virtual | Calls the client item manager derived-state refresh and returns. |
| `009921c0` | `ClientPlayerItemManagerDtor` | ✅ | dtor | Restores the client manager vtable, tears down the base item manager, releases the inventory-derived data-source references, clears vehicle/equipment lists, and calls the generic player item manager destructor. |
| `009924e0` | `ClientPlayerItemManagerScalarDeletingDtor` | ✅ | dtor | Calls the client item manager destructor, conditionally frees the object when the deleting flag is set, and returns the original pointer otherwise. |
