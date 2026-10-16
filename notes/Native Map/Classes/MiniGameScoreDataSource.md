# Class: MiniGameScoreDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c45800` | `MiniGameScoreDataSource_GetColumnName` | ✅ | virtual | Column-name getter of the score table: 0 Name, 1 Icon ID, 2 Score Type, 3 Score Count, 4 Score Max, 5 Score Points, else '<error>'. |
| `00c45e40` | `MiniGameScoreDataSource_GetRowCount` | ✅ | virtual | Row count of the score table: through FUN_009ad160 (which yields the current minigame's score block) it returns the count dword at +0x70, or 0 when there is no score block. |
| `00c462a0` | `MiniGameScoreDataSource_GetValue ⚠downgrade` | 🟢 | virtual | Value getter of the score table: rejects out-of-range rows against the count at score-block+0x70 (score block reached through FUN_009ad160) and returns 1 after writing the requested cell - column 0 = name (string at ele… |
| `00c46fc0` | `MiniGameScoreDataSource_Destruct` | ✅ | dtor | Destructor of the score data source: reinstalls the three MiniGameScoreDataSource vtable pointers and runs the base data-source destructor FUN_0091bf60; nothing else. |
| `00c473a0` | `MiniGameScoreDataSource_Construct` | ✅ | ctor | Constructor of the score data source: registers it under the name BaseClient.MiniGameScore (FUN_00922890), installs the three MiniGameScoreDataSource vtable pointers and stores the two arguments at +0x254/+0x258. |
| `00c47420` | `MiniGameScoreDataSource_ScalarDeletingDtor` | 🟢 | virtual | Virtual slot 0 deleting destructor: runs the score data source destructor 00c46fc0 and frees the object with _free when the low bit of the flag argument is set. |
