# Class: MiniGameGroupDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c45860` | `MiniGameGroupDataSource_GetColumnName` | ✅ | virtual | Column-name getter of the minigame-group table: 32 names from MiniGameId (0) through Price (0x1f), including Name, Description, Icon, LinkName, ShortName, Unlocked, Allowed, Members Only, ParentMiniGamed (verbatim typo … |
| `00c463a0` | `— ⚠reject` | ⚪ | virtual | Value getter of the minigame-group table: rows are param_2*0x70 from the base at *(FUN_009aae50()+0x40); it serves 32 columns - ids/strings straight from the row (0/3 group id, 1/2/4/5/6/8/9/10/12/13/14/16/0x1a/0x1c fie… |
| `00c47030` | `MiniGameGroupDataSource_Destruct` | ✅ | dtor | Destructor of the group data source: reinstalls the three MiniGameGroupDataSource vtable pointers and runs the base data-source destructor FUN_0091bf60. |
| `00c470b0` | `— ⚠reject` | ⚪ | virtual | Row count of the group table: returns the dword at +0x44 of the object obtained from FUN_009aae50 (the group-table singleton). |
| `00c47440` | `MiniGameGroupDataSource_Construct` | ✅ | ctor | Constructor of the group data source: registers it under the name BaseClient.MiniGameGroup (FUN_00922890), installs the three MiniGameGroupDataSource vtable pointers and stores the two arguments at +0x254/+0x258. |
| `00c474c0` | `MiniGameGroupDataSource_ScalarDeletingDtor` | 🟢 | virtual | Virtual slot 0 deleting destructor: runs the group data source destructor 00c47030 and frees the object with _free when the low bit of the flag argument is set. |
