# Class: CharacterCreationStartingSetsDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c2a820` | `CharacterCreationStartingSetsDataSource::GetRowCount` | 🟢 | virtual | Virtual method (vtable slot [2]): loads the starting-sets table object from *(this+0x224)+0x2226c, adjusts this to that object's embedded sub-object at +0x10 and tail-jumps to its vtable slot +0x48 - i.e. it forwards th… |
| `00c2a840` | `CharacterCreationStartingSetsDataSource::GetColumnName` | ✅ | virtual | Virtual column-name accessor (vtable slot [1]): a 14-entry switch returns Id (0), Description (1), RaceId (2), GenderId (3), ModelId (4), SkinToneId (5), HairStyleId (6), HairColorId (7), EyeColorId (8), FacialHairId (9… |
| `00c2a900` | `CharacterCreationStartingSetsDataSource::GetCellText` | 🟢 | virtual | Virtual method (vtable slot [4]) with signature (row, column[, out]): fetches the row record by calling vtable slot +0x44 of the embedded starting-sets table ((*(this+0x224)+0x2226c)+0x10), then switches on the column -… |
| `00c2ac40` | `— ⚠reject` | ⚪ | dtor | Destructor body: resets the three class vtable pointers (this+0, +0x30, +0x34) to CharacterCreationStartingSetsDataSource::vftable and calls the shared base teardown 0091bf60 with the stack cookie, then returns. |
| `00c2acb0` | `CharacterCreationStartingSetsDataSource::CharacterCreationStartingSetsDataSource` | ✅ | ctor | Constructor: registers the data-source name 'BaseClient.StartingSets' through 00922890, installs the three vtable pointers and stores the argument at +0x254, returning this. |
| `00c2ad30` | `CharacterCreationStartingSetsDataSource::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot [0]): calls the destructor body 00c2ac40 and frees the object with _free when the low bit of the flag argument is set. |
