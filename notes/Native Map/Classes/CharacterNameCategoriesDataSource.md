# Class: CharacterNameCategoriesDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c2bb20` | `CharacterNameCategoriesDataSource::GetColumnName` | ✅ | virtual | Virtual column-name accessor (vtable slot [1]): returns CategoryId for column 0, NameCategory for column 1 (a pointer into the literal at 0x0182fed4), NameCategoryDataSourceName for column 2 and <error> otherwise. |
| `00c2bb60` | `— ⚠reject` | ⚪ | virtual | Virtual method (vtable slot [2]) that loads the list pointer from this+0x224 and tail-jumps to 0097d850, which produces the row count. Decoded bytes are mov ecx,[ecx+0x224]; jmp 0097d850. |
| `00c2bb70` | `CharacterNameCategoriesDataSource::GetCellText` | 🟢 | virtual | Virtual method (vtable slot [4]) with signature (row, column[, out]): stores a scratch record, resolves the category through 0097f8a0, then switches on the column - column 0 formats the CategoryId as an integer through … |
| `00c2bd80` | `CharacterNameCategoriesDataSource::~CharacterNameCategoriesDataSource` | 🟢 | dtor | Destructor body: resets the three vtable pointers, releases the embedded IString member at +0x258 (count at +0x264) and calls the shared base teardown 0091bf60, then returns. |
| `00c2be20` | `— ⚠reject` | ⚪ | ctor | Constructor with two arguments: registers a data-source name through 00922890(arg2), installs the three vtable pointers, stores arg2 at +0x254 and initialises the IString member at +0x258 (empty data 01bed558), returnin… |
| `00c2bec0` | `CharacterNameCategoriesDataSource::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot [0]): calls the destructor body 00c2bd80 and frees the object with _free when the low bit of the flag argument is set. Decoded bytes are the standard push esi / mov esi,ecx / call… |
