# Class: MiniGameDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c45a00` | `MiniGameDataSource_GetColumnName` | ✅ | virtual | Column-name getter of the current-minigame table: 17 names - Name, Icon ID, Description, Difficulty, Type, Elapsed Time, Earned Stars, Members Only, Profile Type, Has Action Bar, Total Score, MiniGame Id, IsFlash, IsMic… |
| `00c468c0` | `MiniGameDataSource_GetValue` | ✅ | virtual | Value getter of the current/last-minigame row: when no row handle is supplied it reads the current minigame through FUN_009ad100, or the last played one through FUN_009ad150 when byte +0x230 is set; it then serves the 1… |
| `00c470c0` | `MiniGameDataSource_Destruct` | ✅ | dtor | Destructor of the minigame data source: reinstalls the three MiniGameDataSource vtable pointers, releases the ref-counted goal data source held at +0x25c (virtual slot 10 and FUN_0081ad80 while the engine global is live… |
| `00c474e0` | `MiniGameDataSource_Construct` | ✅ | ctor | Constructor of the live minigame data source: registers it as BaseClient.MiniGame, or as BaseClient.LastMiniGame when the last-flag argument is set; installs the three MiniGameDataSource vtable pointers, stores the two … |
| `00c475c0` | `MiniGameDataSource_ScalarDeletingDtor` | 🟢 | virtual | Virtual slot 0 deleting destructor: runs the minigame data source destructor 00c470c0 and frees the object with _free when the low bit of the flag argument is set. |
