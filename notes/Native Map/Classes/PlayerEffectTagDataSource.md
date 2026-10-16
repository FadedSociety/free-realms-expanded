# Class: PlayerEffectTagDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c8e610` | `PlayerEffectTagDataSource::ColumnName` | ✅ | virtual | Maps columns 0 through 4 to Instance ID, Icon ID, Duration, Time Left, and Name; other values return an error label. |
| `00c8e6b0` | `PlayerEffectTagDataSource::RefreshRows` | 🟢 | method | Refreshes the UI list, walks active effect tags from client state, emits each tag ID, and clears the dirty flag. |
| `00c8e700` | `PlayerEffectTagDataSource::OnRowsChanged` | ✅ | virtual | Refreshes rows when dirty, then invokes FUN_010c8f30. |
| `00c8e720` | `PlayerEffectTagDataSource::GetColumnValue ⚠downgrade` | 🟢 | virtual | Refreshes rows when dirty, resolves the selected tag, and returns instance ID, icon, duration, remaining time, or localized name. |
| `00c8e840` | `PlayerEffectTagDataSource::Dtor` | ✅ | dtor | Restores PlayerEffectTagDataSource vtables and destroys embedded GUI data source state. |
| `00c8e8b0` | `PlayerEffectTagDataSource::PlayerEffectTagDataSource` | ✅ | ctor | Registers BaseClient.PlayerEffectTags, installs source vtables, stores its dependency, and sets the initial dirty state. |
| `00c8e930` | `PlayerEffectTagDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls the source destructor and frees the object when deleting flag bit 0 is set. |
