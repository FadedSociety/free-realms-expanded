# Class: LobbyDataSourceManager::GameListDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a223d0` | `GameListDataSource::CountPlus3` | 🟢 | method | Reads the game object through this+0x224 -> +0x44 -> +0x6f4 and returns 0 when that chain is empty; otherwise calls 0102a5a0 with the game's state field at +0x1a4 (a definition-table lookup that returns an element count… |
| `00a22400` | `GameListDataSource::ValueOrZero ?` | 🟡 | method | When the byte at this+0x228 is clear it calls virtual slot 0x38 on the object at this-0x30; then, if the game chain (this+0x224 -> +0x44 -> +0x6f4) is non-null, tail-jumps to 010c8f30 (returning its result), otherwise r… |
| `00a22470` | `GameListDataSource::ClearFlagAndForward` | 🟢 | method | Stores 0 in the byte at this+0x258 and then tail-calls 010caf20 (a 12-byte function), passing the incoming this through unchanged. |
| `00a22fa0` | `— ⚠reject` | ⚪ | method | Clears the UI list through its virtual slot +0xc, then, when the referenced game object exists, walks the list at game+8 using each node?s +0x14 link. For each node it appends the current index when the node key matches… |
| `00a24a60` | `LobbyDataSourceManager::GameListDataSource::GetColumnValue` | ✅ | virtual | Selects a config identifier by column, resolves later indices, assigns display text and returns its string buffer. |
| `00a25870` | `LobbyDataSourceManager::GameListDataSource::GetItemColumn` | 🟢 | virtual | Returns 1 on its paths. For a valid row and column, writes the selected value or formatted string through param_4; columns 0, 1 and 2 take distinct paths, while later columns use the configured value lookup. |
| `00a26f30` | `LobbyDataSourceManager::GameListDataSource::Dtor` | ✅ | dtor | Restores data-source vtables, releases its shared string and destroys embedded GUI user data. |
| `00a27190` | `LobbyDataSourceManager::GameListDataSource::Ctor` | ✅ | ctor | Registers BaseClient.LobbyGameList, initializes vtables, stores owner and initializes display string storage. |
| `00a27230` | `LobbyDataSourceManager::GameListDataSource::ScalarDeletingDtor` | ✅ | virtual | Calls the data-source destructor and frees object when deletion flag bit zero is set. |
