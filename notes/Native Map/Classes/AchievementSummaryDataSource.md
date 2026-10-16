# Class: AchievementSummaryDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ce5b80` | `AchievementSummaryDataSource::ColumnName ?` | 🟡 | virtual | Maps indices 0 through 4 to Id, Name, Total, Completed, and cColumnPercentage; returns <error> otherwise. |
| `00ce5cb0` | `AchievementSummaryDataSourceEntry::Reserve ?` | 🟡 | method | Chooses element capacity from current and requested counts, writes capacity through param_3, returns old storage or allocates count times 0x40 bytes. |
| `00ce5d50` | `AchievementSummaryDataSourceEntry::Free ?` | 🟡 | function | Passes the supplied pointer and alignment 4 to the memory free routine. |
| `00ce5d70` | `AchievementSummaryDataSourceEntry::MoveOrDestroy ?` | 🟡 | method | When flag is set calls an entry virtual method; otherwise swaps its pointer and two counters with the supplied entry. |
| `00ce6800` | `AchievementSummaryDataSource::Refresh ?` | 🟡 | virtual | Scans active achievements, counts matching entries, sums their associated values, sends total points to the Scaleform handler, refreshes source, and sets its flag. |
| `00ce6ae0` | `AchievementSummaryDataSource::GetRowCount ?` | 🟡 | virtual | Refreshes the summary source and returns the dword at object offset +0x238. |
| `00ce6b00` | `AchievementSummaryDataSource::EntryArrayDtor ?` | 🟡 | dtor | Restores embedded array vtable, destroys entries by count, frees backing storage, and clears the storage pointer. |
| `00ce6b30` | `AchievementSummaryDataSource::EntryArrayScalarDeletingDtor ?` | 🟡 | dtor | Restores entry array vtable, destroys entries, frees backing storage, then conditionally frees the array object. |
| `00ce6b90` | `AchievementSummaryDataSourceEntryAt ?` | 🟡 | function | For index at or beyond count obtains an initialized entry; returns base pointer plus index times 0x40. |
| `00ce6bc0` | `AchievementSummaryDataSource::DtorBody ?` | 🟡 | dtor | Restores summary source vtables, destroys entry array elements/storage, and calls the embedded base holder destructor. |
| `00ce6c70` | `— ⚠reject` | ⚪ | virtual | Returns current entry id, localized name, total, completed count, or integer percentage for columns 0..4. |
| `00ce6db0` | `AchievementSummaryDataSource::Ctor ?` | 🟡 | ctor | Constructs BaseClient.Achievements.Summary holder, installs source vtables, stores arguments, clears flags, and initializes entry array. |
| `00ce6e50` | `AchievementSummaryDataSource::ScalarDeletingDtor ?` | 🟡 | dtor | Calls summary source destructor body, conditionally frees object when flag bit 0 is set, and returns its pointer. |
