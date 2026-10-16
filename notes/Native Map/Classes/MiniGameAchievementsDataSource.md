# Class: MiniGameAchievementsDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c45b90` | `MiniGameAchievementsDataSource_PopulateOnDemand` | 🟢 | virtual | Virtual slot 2: when the loaded byte at +0x248 is zero it first calls base vtable slot 15 (offset 0x3c) on the subobject at this-0x30, and then tail-jumps to FUN_010c8f30 with this+4 in ECX (the base implementation of t… |
| `00c45bb0` | `MiniGameAchievementsDataSource_GetColumnName` | ✅ | virtual | Column-name getter of the trophies table: 0 Id, 1 Name, 2 Description, 3 Icon ID, else '<error>'. |
| `00c45f80` | `MiniGameAchievementsDataSource_PushAllRows` | 🟢 | virtual | Writes every achievement row into the table sink at +0x34: calls the sink's virtual slot 3 (begin update) once, then for each node of the list at +0x264 (next at +4) calls the sink's virtual slot 1 with the node value, … |
| `00c46e40` | `MiniGameAchievementsDataSource_GetValue` | ✅ | virtual | Value getter of the trophies table: prefills the out buffer with the wide string '<not found>', resolves the trophy object chain through FUN_008a6760 and FUN_00baa420, and serves 0 id, 1/2 name and description strings (… |
| `00c47220` | `— ⚠reject` | ⚪ | ctor | Constructor variant of the achievements data source: installs the three MiniGameAchievementsDataSource vtable pointers, runs FUN_00728070 (initialise the achievements store) and then the base data-source constructor FUN… |
| `00c472a0` | `MiniGameAchievementsDataSource_SetSource` | 🟢 | virtual | Virtual slot 14: hands the argument to FUN_009b5140 (install the achievements source), clears the populated byte at +0x278 and then invokes virtual slot 4 of this object through the vtable pointer (a bare tail call with… |
| `00c47680` | `MiniGameAchievementsDataSource_ConstructTrophies` | ✅ | ctor | Constructor of the trophies variant: registers the instance under the name BaseClient.MiniGame.Trophies (FUN_00922890), stores the arguments at +0x254/+0x258, installs the three MiniGameAchievementsDataSource vtable poi… |
| `00c47720` | `MiniGameAchievementsDataSourceA_ScalarDeletingDtor` | 🟢 | virtual | Virtual slot 0 deleting destructor of the 00c47220 constructor variant: runs that destructor body and frees the object with _free when the low bit of the flag argument is set. |
