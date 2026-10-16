# Class: MiniGameGoalDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c45760` | `MiniGameGoalDataSource_GetColumnName` | ✅ | virtual | Column-name getter of the goal table data source: index 0 ID, 1 Name, 2 Description, 3 Bonus, 4 Status Text, 5 State, 6 Reward Stars, 7 MembersOnly, 8 CategoryPrefix, anything else '<error>'. |
| `00c45e20` | `MiniGameGoalDataSource_GetRowCount` | ✅ | virtual | Row count of the goal table: resolves the manager through *(this+0x228), asks it for the current minigame with FUN_009ad100(0) and returns that minigame's goal count dword at +0x84, or 0 when there is no active minigame. |
| `00c460e0` | `MiniGameGoalDataSource_GetValue` | ✅ | virtual | Value getter of the goal table: refuses column >= 9 or a missing current minigame (FUN_009ad100(0)); refreshes the index list when byte +0x23c is set (00c45fc0); resolves the row key through FUN_0079db00 and turns it in… |
| `00c46f30` | `MiniGameGoalDataSource_Destruct` | ✅ | dtor | Destructor of the goal data source: reinstalls the three MiniGameGoalDataSource vtable pointers, installs the SoeUtil::List<int,-1> vtable for the index list at +0x25c, runs FUN_007798c0 and finally the base data-source… |
| `00c472d0` | `MiniGameGoalDataSource_Construct` | ✅ | ctor | Constructor of the goal data source: registers the instance under the data-source name BaseClient.MiniGameGoals (FUN_00922890), installs the three MiniGameGoalDataSource vtable pointers, stores the two constructor argum… |
| `00c47380` | `MiniGameGoalDataSource_ScalarDeletingDtor` | 🟢 | virtual | Virtual slot 0 deleting destructor: runs the goal data source destructor 00c46f30 and frees the object with _free when the low bit of the flag argument is set. |
