# Class: GroupMemberDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a7c810` | `GroupMemberDataSource::GetColumnName` | ✅ | virtual | Pure column-name table for the group member list: returns PlayerName, PlayerGuid, PlayerJobId, PlayerJobBackgroundIconId, PlayerJobIconId, PlayerJobRank, PlayerStatusMessage, IsGroupLeader, InSameMinigame for indices 0.… |
| `00a7c9d0` | `GroupMemberDataSource::GetRowCount` | 🟢 | virtual | Reads the group pointer at this+0x224 (primary object +0x254) and, when its field +0x6ec is non-null, returns the count stored at group+0xc; when there is no group it returns the constant 1. |
| `00a7c9f0` | `GroupMemberDataSource::RefreshMemberRows` | 🟢 | method | Rebuilds the member rows of the object at +0: calls virtual slot 3 (offset 0xc) and then slot 1 (offset 4) with 0 on the row-sink object at +0x34 to clear it, resolves the group through (this+0x254)->+0x6ec and, when it… |
| `00a7cd60` | `GroupMemberDataSource::GetColumnValue` | ✅ | virtual | Row-value getter of the group member list (vtable slot 4). When the byte at +0x228 is clear it first rebuilds the rows through 00a7c9f0, resolves the group object at (this+0x224)+0x6ec and the member record through the … |
| `00a7db00` | `GroupMemberDataSource::~GroupMemberDataSource` | ✅ | dtor | Destructor body: rewrites the three GroupMemberDataSource vtable pointers (at +0, +0x30 and +0x34) and then runs the shared base destructor FUN_0091bf60. |
| `00a7dbe0` | `GroupMemberDataSource::GroupMemberDataSource` | ✅ | ctor | Constructor: registers the name BaseClient.GroupMemberData through FUN_00922890, installs the three class vtables (+0, +0x30, +0x34), stores the owner argument at +0x254 and clears the changed byte at +0x258. |
| `00a7dc60` | `GroupMemberDataSource::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (slot 0 of the class vtable): calls the destructor body 00a7db00 and, when bit 0 of param_2 is set, releases the object with _free, returning the object pointer. |
