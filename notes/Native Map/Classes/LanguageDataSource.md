# Class: LanguageDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a852a0` | `— ⚠reject` | ⚪ | virtual | Refreshes the language table (vtable slot [14]): triggers the underlying accessor (this+0x34 slot 3), then for every entry of the global language list (00769710 -> +4 -> +0x1e4, next link at +0x84) checks whether any al… |
| `00a85660` | `LanguageDataSourceDtor` | ✅ | dtor | Destructor body of the language data source: stores the three LanguageDataSource vtables and then runs the shared locale data-source destructor 00a855b0. |
| `00a85860` | `LanguageDataSourceCtor` | ✅ | ctor | Constructor of the BaseClient.Language data source: runs the shared locale constructor 00a857a0 with the owner pointer and the name "BaseClient.Language", then overwrites the vtable fields at +0, +0x30 and +0x34 with th… |
| `00a858e0` | `LanguageDataSourceScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot [0]) of LanguageDataSource: runs the destructor body 00a85660 and frees the object with _free when the delete flag bit 0 is set. |
