# Class: CharacterNamesDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c2b200` | `CharacterNamesDataSource::GetColumnName` | ✅ | virtual | Virtual column-name accessor (vtable slot [1]): returns NameId for column 0 (a pointer into the literal at 0x0182fd14), Name for column 1, LocalizedName for column 2 and <error> otherwise. |
| `00c2b240` | `CharacterNamesDataSource::GetRowCount` | 🟢 | virtual | Virtual method (vtable slot [2]): prepares a 52-byte scratch record through 0070bcc0(*(this+0x22c)), queries the count with 0097e590(scratch), releases with 00703350 and returns the queried dword. |
| `00c2b2c0` | `CharacterNamesDataSource::GetCellText` | 🟢 | virtual | Virtual method (vtable slot [4]) with signature (row, column, out[, extra]): builds a scratch record (00703540/00703350) and resolves the row through the name-list helpers 0097f8e0 and 0097a510/009b0e90 (with the client… |
| `00c2b550` | `CharacterNamesDataSource::~CharacterNamesDataSource` | 🟢 | dtor | Destructor body: resets the three vtable pointers, releases the embedded IString member at +0x258 (count at +0x264) and calls the shared base teardown 0091bf60, then returns. |
| `00c2b5f0` | `— ⚠reject` | ⚪ | ctor | Constructor with three arguments: registers a data-source name through 00922890(arg3), installs the three vtable pointers, stores arg2 at +0x254, initialises the IString member at +0x258 (empty data 01bed476) and assign… |
| `00c2b690` | `CharacterNamesDataSource::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot [0]): calls the destructor body 00c2b550 and frees the object with _free when the low bit of the flag argument is set. |
