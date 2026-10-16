# Class: RecipeDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c8a5b0` | `RecipeDataSource::ColumnName` | ✅ | virtual | Returns labels for columns 0 through 9 and Has Component for valid extended columns; invalid indices return an error label. |
| `00c8a670` | `RecipeDataSource::VirtualSlot6Adjustor` | ✅ | virtual | Adjusts ECX by -0x254, loads a function pointer from vtable offset 0x10, and jumps through it. |
| `00c8aa10` | `RecipeDataSource::GetCount` | ✅ | virtual | Returns the dword stored at object offset 0x884. |
| `00c8b360` | `RecipeDataSource::GetColumnValue` | ✅ | virtual | Resolves a recipe row and provides ID, name, icon, description, member flag, component count, filter, tint, item-definition, requirement, and component status values. |
| `00c8cc60` | `RecipeDataSource::Dtor` | ✅ | dtor | Restores RecipeDataSource vtables, releases a retained reference, destroys two hash maps and embedded GUI data table state. |
| `00c8cd60` | `RecipeDataSource::ScalarDeletingAdjustor` | ✅ | dtor | Subtracts 0x254 from ECX and jumps to 0x00c8d0d0; this is an adjustor thunk, not the destructor body. |
| `00c8cd70` | `— ⚠reject` | ⚪ | method | Clears pending rows, replaces the nested list when ID is nonpositive, or removes rows whose field +0x24 matches; then notifies virtual slot 0x10. |
| `00c8cde0` | `— ⚠reject` | ⚪ | method | Clears pending rows, replaces the nested list when ID is nonpositive, or removes rows whose field +0x2c matches; then notifies virtual slot 0x10. |
| `00c8cfc0` | `RecipeDataSource::RecipeDataSource` | ✅ | ctor | Registers BaseClient.Recipes, initializes its data source and two hash lists, creates a component source, stores references, and registers inventory data. |
| `00c8d0d0` | `RecipeDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls RecipeDataSource cleanup and frees the object when deleting flag bit 0 is set. |
