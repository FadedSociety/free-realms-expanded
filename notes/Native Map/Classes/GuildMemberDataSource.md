# Class: GuildMemberDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cb5450` | `GuildMemberDataSource::ColumnName` | ✅ | virtual | Maps column numbers to guild member identity, rank, online state, profile values, encounter state, location and friend status labels. |
| `00cb5720` | `GuildMemberDataSource::GetRowCount` | ✅ | virtual | Returns zero if this+0x228 is null; otherwise returns the pointed-to field at offset +0xa0. |
| `00cb6260` | `GuildMemberDataSource::Ctor` | ✅ | ctor | Initializes the BaseClient.GuildMembers holder, installs class vtables, stores the source, and initializes the embedded character-GUID array and dirty state. |
| `00cb6320` | `GuildMemberDataSource::Dtor` | ✅ | dtor | Clears the GUID array count, restores its array vtable, frees aligned storage, and destroys the embedded GUI user-data holder. |
| `00cb63b0` | `GuildMemberDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls the guild-member data-source destructor and frees the object when the deleting flag's low bit is set. |
| `00cb63d0` | `MemberStubListRebuildRows` | 🟢 | method | Clears the current character-GUID row array, traverses the current member source, builds and sorts two temporary MemberStub lists, copies their key pairs into the row array, and clears the dirty byte. |
| `00cb65f0` | `GuildMemberDataSource::GetCellValue ⚠downgrade` | 🟢 | virtual | Validates and resolves a row's GUID, then maps column indices to identity, profile, member state, rank and permission values, including comparisons with current member context. |
| `00cb6b80` | `— ⚠reject` | ⚪ | method | For special update types, delegates to the full member update path; otherwise validates guild context, resolves the incoming member record and updates the matching field. |
