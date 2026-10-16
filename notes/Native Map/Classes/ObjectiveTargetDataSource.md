# Class: ObjectiveTargetDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cb8480` | `ObjectiveTargetDataSource::ColumnHeaderText` | ✅ | virtual | Virtual slot 1 of ObjectiveTargetDataSource: switches on the index argument and returns a static header string for 0 Active, 1 Guid, 2 Name, 3 Location X, 4 Location Z, 5 Zone ID, 6 Target Type, 7 Screen Angle; any othe… |
| `00cb8810` | `ObjectiveTargetDataSource::GetCellValue` | 🟢 | virtual | Virtual slot 4 of ObjectiveTargetDataSource (thiscall; param_2 float, param_3 column, param_4 output). Returns 0 unless param_2 == 0.0. Column 0 passes one of two static strings (DAT_0180c498 or DAT_017f7d9c, chosen by … |
| `00cb89a0` | `ObjectiveTargetDataSource::Ctor` | ✅ | ctor | Thiscall with one dword argument: calls 00922890 with the BaseClient.ObjectiveTarget string, stores ObjectiveTargetDataSource::vftable at p+0, p+0x30 and p+0x34, stores the argument at p+0x254, clears the byte at p+0x25… |
| `00cb8b40` | `ObjectiveTargetDataSource::DeletingDtor` | ✅ | dtor | Virtual slot 0 of ObjectiveTargetDataSource (thiscall, one byte flag): calls the dtor body 00cb8ab0 on the receiver, then when flag bit 0 is set calls _free (thunk 00d82463) on it; returns the receiver. |
