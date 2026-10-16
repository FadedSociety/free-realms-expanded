# Class: LocaleDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a85140` | `LocaleLanguageDataSource::GetColumnName` | ✅ | virtual | Column-title getter (vtable slot [1] of both LocaleDataSource and LanguageDataSource): switches on the column index and returns the literal column names "ID" (0), "LanguageID" (1), "Location" (2), "Language" (3), "Locat… |
| `00a851d0` | `LocaleLanguageDataSource::GetRowCount` | ✅ | virtual | If the loaded flag at this+0x238 is clear, invokes the sibling LocaleDataSource method at vtable byte offset +0x38 (slot 14); then tail-jumps to 010c8f30 for the row count. |
| `00a85240` | `— ⚠reject` | ⚪ | virtual | Refreshes the locale table (vtable slot [14]): triggers the underlying table accessor through this+0x34 slot 3, walks the global locale list (00769710 -> +4 -> +0x1e4 -> list head at +0xc, next link at +0x84) and adds e… |
| `00a85370` | `LocaleLanguageDataSource::GetField` | ✅ | virtual | Lua/UI field getter (vtable slot [4] of LocaleDataSource and LanguageDataSource): if this+0x238 is clear, invokes the LocaleDataSource method at the sibling subobject vtable byte offset +0x38 (slot 14); obtains row para… |
| `00a855b0` | `LocaleDataSourceDtor` | ✅ | dtor | Destructor body of the locale data source: stores the three LocaleDataSource vtables, frees the int array owned by the member at param_1[0x95] (SoeUtil::Array<int,0,1> vtable, buffer at [0x96] freed with FreeMemory alig… |
| `00a856c0` | `— ⚠reject` | ⚪ | ctor | Constructor of the BaseClient.Locale data source: registers the name through 00922890(PTR_s_BaseClient.Locale), stores the three LocaleDataSource vtables, initialises the int array member at [0x95] (vtable plus zeroed f… |
| `00a85780` | `LocaleDataSourceScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot [0]) of LocaleDataSource: runs the destructor body 00a855b0 and frees the object with _free when the delete flag bit 0 is set. |
| `00a857a0` | `— ⚠reject` | ⚪ | ctor | Shared constructor body of the locale/language data sources: registers the caller-supplied name (param_3) through 00922890, stores the three LocaleDataSource vtables, initialises the int array member at [0x95], saves th… |
