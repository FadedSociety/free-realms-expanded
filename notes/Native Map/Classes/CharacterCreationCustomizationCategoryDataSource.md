# Class: CharacterCreationCustomizationCategoryDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c2ad60` | `CharacterCreationCustomizationCategoryDataSource::GetColumnName` | ✅ | virtual | Virtual column-name accessor (vtable slot [1]): returns CategoryId (0), CategoryTitleId (1), DataSourceNameSuffix (2), DataSourceName (3, the DataSourceName literal at 0x0182fc4c), Race (4) and Gender (5), with <error> … |
| `00c2ae40` | `CharacterCreationCustomizationCategoryDataSource::GetRowCount` | 🟢 | virtual | Virtual method (vtable slot [2]): returns the dword at +0x21c of the object referenced by this+0x224 - the number of customization categories. Decoded bytes are mov eax,[ecx+0x224]; mov eax,[eax+0x21c]; ret. |
| `00c2ae60` | `— ⚠reject` | ⚪ | virtual | Virtual method (vtable slot [4]) with signature (row, column, out[, extra]): walks the category list from this+0x224 (root at +0x214, next at +0x24) to the requested row, then switches on the column - 0 CategoryId and 1… |
| `00c2b090` | `CustomizationCategoryDataSourceDtor` | 🟢 | dtor | Destructor body of CharacterCreationCustomizationCategoryDataSource: resets the three vtable pointers, releases the embedded SoeUtil::IString member at +0x258 (reference count at +0x264, release through vftable[2]) and … |
| `00c2b130` | `CustomizationCategoryDataSourceCtor` | ✅ | ctor | Constructor of CharacterCreationCustomizationCategoryDataSource: registers the data-source name 'BaseClient.CharacterCustomizationCategory' through 00922890, installs the three vtable pointers, stores the argument at +0… |
| `00c2b1d0` | `CharacterCreationCustomizationCategoryDataSource::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot [0]): calls the destructor body 00c2b090 and frees the object with _free when the low bit of the flag argument is set. |
