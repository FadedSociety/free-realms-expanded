# Class: ObjectiveHelperDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cb7710` | `ObjectiveHelperDataSource::ColumnHeaderText` | ✅ | virtual | Virtual slot 1 of ObjectiveHelperDataSource: switches on the index argument and returns a static header string for 0 ID, 1 Text, 2 Icon ID, 3 Is Bonus, 4 Status, 5 MembersOnly, 6 Category Prefix, 7 Goal Count, 8 Goal To… |
| `00cb7800` | `ObjectiveHelperDataSource::GetField230 ?` | 🟡 | virtual | Virtual slot 2 of ObjectiveHelperDataSource: bytes 8b 81 30 02 00 00 c3 decode to mov eax,[ecx+0x230]; ret, so it returns the dword at receiver+0x230. |
| `00cb7cf0` | `ObjectiveHelperDataSource::GetCellValue` | 🟢 | virtual | Virtual slot 4 of ObjectiveHelperDataSource (thiscall: row index, column, output). Row record = receiver+0x22c base plus index*0x2c. Switch on column 0..8: 0 passes the record to 009969b0; 1 calls 00a8b9a0 with the outp… |
| `00cb8320` | `ObjectiveHelperDataSource::DtorBody` | ✅ | dtor | Fastcall on receiver p: stores ObjectiveHelperDataSource::vftable at p+0, p+0x30 and p+0x34, then the Array<ObjectiveEntry> vftable at p+0x258; calls 00cb7aa0 on the array header with count p+0x260 (truncates to zero re… |
| `00cb83d0` | `ObjectiveHelperDataSource::Ctor ⚠downgrade` | 🟢 | ctor | Thiscall with one dword argument: calls 00922890 with the BaseClient.ObjectiveHelper string, stores ObjectiveHelperDataSource::vftable at p+0, p+0x30 and p+0x34, stores the argument at p+0x254, stores Array<ObjectiveEnt… |
| `00cb8460` | `ObjectiveHelperDataSource::DeletingDtor` | ✅ | dtor | Virtual slot 0 of ObjectiveHelperDataSource (thiscall, one byte flag): calls the dtor body 00cb8320 on the receiver, then when flag bit 0 is set calls _free (thunk 00d82463) on it; returns the receiver and pops 4. Bytes… |
