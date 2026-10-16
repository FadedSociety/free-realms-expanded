# Class: CharacterCreationCustomizationDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c29800` | `CharacterCreationCustomizationDataSource::GetColumnName` | ✅ | virtual | Virtual column-name accessor (vtable slot [1]): a switch returns Guid (0), IconId (1), IconTint (2), ItemId (3), Race (4), Gender (5), Name (6) and Description (7), with <error> for other indices. |
| `00c29880` | `CharacterCreationCustomizationDataSource::GetRowCount` | 🟢 | virtual | Virtual method (vtable slot [2]): loads the row-list pointer from this+0x224 and the key from this+0x228 and tail-calls the hash-map lookup 0097d710, returning its result (the value stored at entry+0xc for the matching … |
| `00c298d0` | `— ⚠reject` | ⚪ | virtual | Virtual method (vtable slot [4]) with signature (row, column[, out]): resolves the row record with the keyed lookup 0097d750(this+0x228, row) and then switches on the column - 0 formats the 64-bit Guid through 00c298a0,… |
| `00c29b10` | `CharacterCreationCustomizationDataSource::~CharacterCreationCustomizationDataSo…` | 🟢 | dtor | Destructor body: resets the three vtable pointers (this+0, +0x30, +0x34), releases the embedded SoeUtil::IString at this+0x25c (checks the reference count at this+0x268 and calls vftable[2] when it drops to zero or belo… |
| `00c29bb0` | `CharacterCreationCustomizationDataSource::CharacterCreationCustomizationDataSou…` | 🟢 | ctor | Constructor with three arguments: registers a data-source name through 00922890(arg4), installs the three vtable pointers, stores arg2 at +0x254 and arg3 at +0x258, initialises the embedded IString member at +0x25c (vft… |
| `00c29c50` | `CharacterCreationCustomizationDataSource::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot [0]): calls the destructor body 00c29b10 and frees the object with _free when the low bit of the flag argument is set. |
