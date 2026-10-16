# Class: MiniGameDataDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c45af0` | `MiniGameDataDataSource_GetColumnName` | ✅ | virtual | Column-name getter of the minigame-definition table: 11 names - Id, Type, Name, Description, Icon ID, Difficulty, Members Only, GroupId, GroupName, GroupIcon, StartSoundId - else '<error>'. |
| `00c46c20` | `MiniGameDataDataSource_GetValue` | ✅ | virtual | Value getter of the minigame-definition table: resolves the row through FUN_009ad1b0 and serves 11 columns - 0 id (+0x10), 1 type (+0x14 written through FUN_00a81110), 2/3 name and description (strings at +0x28/+0x2c wr… |
| `00c47180` | `MiniGameDataDataSource_Destruct` | ✅ | dtor | Destructor of the minigame-data data source: reinstalls the three MiniGameDataDataSource vtable pointers and runs the base data-source destructor FUN_0091bf60. |
| `00c47210` | `MiniGameDataDataSource_GetRowCount` | ✅ | virtual | Row count of the minigame-definition table: returns the dword at +0x2bf3c of the table object reached through *(this+0x228). |
| `00c475e0` | `MiniGameDataDataSource_Construct` | ✅ | ctor | Constructor of the minigame-data data source: registers it under the name BaseClient.MiniGameData (FUN_00922890), installs the three MiniGameDataDataSource vtable pointers and stores the two arguments at +0x254/+0x258. |
| `00c47660` | `MiniGameDataDataSource_ScalarDeletingDtor` | 🟢 | virtual | Virtual slot 0 deleting destructor: runs the minigame-data destructor 00c47180 and frees the object with _free when the low bit of the flag argument is set. |
