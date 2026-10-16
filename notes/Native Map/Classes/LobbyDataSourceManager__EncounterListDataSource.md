# Class: LobbyDataSourceManager::EncounterListDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a224a0` | `EncounterListDataSource::ColumnName` | ✅ | method | Returns the literal column-name string for the index argument when the encounter chain (this+0x224 -> +0x44 -> +0x6f4) is non-null: 0 "GameId", 1 "NumberOfPlayers", 2 "OwnerName", 3 "EncounterName", 4 "Difficulty", 5 "T… |
| `00a22520` | `EncounterListDataSource::ValueOrZero ? ⚠downgrade` | 🟡 | method | Identical in shape to the game-list variant: if this+0x228 is clear it calls virtual slot 0x38 on the object at this-0x30, then returns 0 when the encounter chain (this+0x224 -> +0x44 -> +0x6f4) is empty or tail-jumps t… |
| `00a22560` | `EncounterListDataSource::ClearFlagAndForward` | 🟢 | method | Stores 0 in the byte at this+0x258 and tail-calls 010caf20, forwarding this; byte-for-byte the same shape as the game-list variant at 00a22470. |
| `00a23940` | `EncounterListDataSource::RefreshRows` | ✅ | method | Clears the UI list through virtual slot 0xc of the object at this+0x34 and, when the encounter object (this+0x254 -> +0x44 -> +0x6f4) exists, walks the list at this+0x254+0xf8 (next through each node's +0x2d0) appending… |
| `00a25a40` | `LobbyDataSourceManager::EncounterListDataSource::GetItemColumn` | ✅ | virtual | Returns 1 on its paths. For a found encounter row, writes the requested column value through param_4: column 0 writes the row value, column 1 sums linked-entry fields, columns 2 and 5 format strings, and columns 3 and 4… |
| `00a26fd0` | `LobbyDataSourceManager::EncounterListDataSource::Dtor` | ✅ | dtor | Restores data-source vtables, releases its shared string and destroys embedded GUI user data. |
| `00a27250` | `LobbyDataSourceManager::EncounterListDataSource::Ctor` | ✅ | ctor | Registers BaseClient.EncounterGameList, initializes vtables, stores owner and initializes display string storage. |
| `00a272f0` | `LobbyDataSourceManager::EncounterListDataSource::ScalarDeletingDtor` | ✅ | virtual | Calls data-source destructor and frees object when deletion flag bit zero is set. |
