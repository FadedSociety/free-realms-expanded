# Class: ClientActionBarDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cf3b20` | `ClientActionBarDataSource::GetFieldName` | ✅ | virtual | Maps field ids 0 through 8 to Is Empty, Icon ID, Name, Is Enabled, Refresh Time Left, Total Refresh Time, Quantity, Is Targeting, and Icon Tint Value; returns <error> for other ids. |
| `00cf3bb0` | `ClientActionBarDataSource::GetCount` | ✅ | virtual | Returns the dword at the nested object offset this+0x224+0x18. |
| `00cf3bc0` | `ClientActionBarDataSource::GetFieldValue` | ✅ | virtual | Finds an action-bar slot by id in an eight-bucket map and returns selected fields for ids 0 through 8, including empty, icon id, name, enabled, refresh, quantity, targeting and tint; returns zero when the slot is absent. |
| `00cf3d60` | `ClientActionBarDataSource::Dtor` | ✅ | dtor | Restores the ClientActionBarDataSource vtable at object/base offsets and destroys the embedded GUI datasource base through 0091bf60. |
| `00cf3dd0` | `ClientActionBarDataSource::Ctor` | ✅ | ctor | Constructs the GUI datasource base, installs the ClientActionBarDataSource vtable at its base offsets, stores the owner pointer at +0x254, and returns this. |
| `00cf3e50` | `ClientActionBarDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls the ClientActionBarDataSource destructor body, frees the object when the low bit of the flag is set, and returns the object otherwise. |
