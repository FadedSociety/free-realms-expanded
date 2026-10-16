# Class: CustomizationItemDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ce7b50` | `CustomizationItemDataSource::Ctor ?` | 🟡 | ctor | Constructs the BaseClient.CustomizationItemDataSource holder, installs the customization source vtables, and stores the supplied manager pointer. |
| `00ce7be0` | `CustomizationItemDataSource::DtorBody ?` | 🟡 | dtor | Restores customization source vtables at the object and two embedded bases, then destroys the embedded base holder. |
| `00ce7c80` | `CustomizationItemDataSource::ColumnName ?` | 🟡 | virtual | Maps indices 0 and 1 to Id and ItemId; for other indices returns the result of a fallback helper. |
| `00ce7cf0` | `CustomizationItemDataSource::ScalarDeletingDtor ?` | 🟡 | dtor | Calls customization source destructor body, conditionally frees the object when flag bit 0 is set, and returns its pointer. |
| `00ce7d10` | `CustomizationItemDataSource::GetRowCount ?` | 🟡 | virtual | Reads manager pointer through object fields and returns the dword at manager-relative offset 0x354f0. |
| `00ce7da0` | `CustomizationItemDataSource::GetColumn ?` | 🟡 | virtual | If refresh flag is clear, refreshes first; returns a row-not-found marker when indexed access fails, otherwise exposes id, item id, or formatted item data. |
