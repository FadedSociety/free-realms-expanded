# Class: CharacterTypesDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c294a0` | `CharacterTypesDataSource::GetColumnName` | ✅ | virtual | Virtual column-name accessor (primary vtable slot [1] of CharacterTypesDataSource): a switch over the column index returns the literals CharacterTypeId (0), CharacterTypeNameStringId (1), CharacterRace (2) and Character… |
| `00c29560` | `CharacterTypesDataSource::GetRowCount` | 🟢 | virtual | Virtual method (primary vtable slot [2]): returns the dword at offset +0xc of the row-list object held at this+0x224, i.e. the row count of the data source. |
| `00c29570` | `— ⚠reject` | ⚪ | virtual | Virtual method (primary vtable slot [4]) with signature (row, column, outString): walks the singly linked row list from this+0x224 (first node at +4, next node at node+0x70) until it reaches the requested row, then swit… |
| `00c296f0` | `— ⚠reject` | ⚪ | dtor | Destructor body: resets the three class vtable pointers at this+0, this+0x30 and this+0x34 to CharacterTypesDataSource::vftable and calls the base cleanup 0091bf60 with the stack cookie, then returns. |
| `00c29750` | `CharacterTypesDataSource::CharacterTypesDataSource` | ✅ | ctor | Constructor: registers the data-source name 'BaseClient.CharacterTypes' through 00922890, installs the three vtable pointers at this+0, this+0x30 and this+0x34, stores the single argument at this+0x254 and returns this. |
| `00c297d0` | `CharacterTypesDataSource::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot [0] of the class): calls the destructor body 00c296f0 and, when the low bit of the flag argument is set, frees the object with _free, then returns the object pointer. |
