# Class: RecipeComponentDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c8a500` | `RecipeComponentDataSource::ColumnName` | ✅ | virtual | Maps indices 0 through 6 to Name, Icon ID, Description, Required Count, Count in Inventory, Tint Value, and Item Definition Id; other indices return an error label. |
| `00c8a570` | `RecipeComponentDataSource::VirtualSlot6Adjustor` | ✅ | virtual | Adjusts ECX by -0x254, loads a function pointer from vtable offset 0x10, and jumps through it. |
| `00c8a9f0` | `RecipeComponentDataSource::GetItemDefinitionId` | ✅ | virtual | Returns zero when field +0x248 is null; otherwise returns the dword at row pointer +0x3c. |
| `00c8b1d0` | `— ⚠reject` | ⚪ | virtual | Resolves an indexed component row and returns requested ID, name, icon, count, tint, item-definition, or requirement values through the UI value interface. |
| `00c8bad0` | `RecipeComponentDataSource::Dtor` | ✅ | dtor | Restores four RecipeComponentDataSource vtables, destroys embedded source state and the base data-source object. |
| `00c8bb70` | `RecipeComponentDataSource::ScalarDeletingDtorAdjustor` | ✅ | dtor | Subtracts 0x254 from ECX and jumps to 0x00c8bdd0; this is an adjustor thunk, not the destructor body. |
| `00c8bd10` | `RecipeComponentDataSource::RecipeComponentDataSource` | ✅ | ctor | Registers BaseClient.RecipeComponents, initializes inherited data-source state and stores the supplied dependencies and inventory key. |
| `00c8bdd0` | `RecipeComponentDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls the component source destructor, then frees the object when deleting flag bit 0 is set. |
