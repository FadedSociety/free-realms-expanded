# Class: CharacterCreationPresetDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c2b6c0` | `CharacterCreationPresetDataSource::GetColumnName` | ✅ | virtual | Virtual column-name accessor (vtable slot [1]): returns Id (0), Description (1), ModelId (2), ItemGroupId (3), Gender (4), HeadId (5), HairId (6), HairTintId (7), EyeTintId (8), SkinToneId (9), FaceId (10), FacialHairId… |
| `00c2b780` | `— ⚠reject` | ⚪ | virtual | Virtual method (vtable slot [2]) that only loads the row-list pointer from this+0x224 and tail-jumps to 0097d970 - the row count comes from the preset-list helper. Decoded bytes are mov ecx,[ecx+0x224]; jmp 0097d970. |
| `00c2b790` | `CharacterCreationPresetDataSource::GetCellText` | 🟢 | virtual | Virtual method (vtable slot [4]) with signature (row, column[, out]): resolves the preset record with 0097d990(row) and switches on the column - 0 formats the record id as an integer (009969b0), 1 copies the Description… |
| `00c2ba10` | `CharacterCreationPresetDataSource::~CharacterCreationPresetDataSource` | 🟢 | dtor | Destructor body: resets the three vtable pointers and calls the shared base teardown 0091bf60, then returns. |
| `00c2ba70` | `CharacterCreationPresetDataSource::CharacterCreationPresetDataSource` | ✅ | ctor | Constructor: registers the data-source name 'BaseClient.CharacterPresets' through 00922890, installs the three vtable pointers and stores the argument at +0x254, returning this. |
| `00c2baf0` | `CharacterCreationPresetDataSource::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot [0]): calls the destructor body 00c2ba10 and frees the object with _free when the low bit of the flag argument is set. Decoded bytes are the standard push esi / mov esi,ecx / call… |
