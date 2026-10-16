# Class: MatchmakingAvailableBuddiesDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a7f4c0` | `MatchmakingAvailableBuddiesDataSource::GetColumnName` | ✅ | virtual | Column-name table of the available-buddies list: BuddyName for index 0, BuddyGuid for index 1 and <error> otherwise; it never touches this. |
| `00a82590` | `— ⚠reject` | ⚪ | method | Rebuild routine for the available-buddies table: clears the node list at this+0x260, then for every row of the UI data source "BaseClient.GuildMembers" (row count via its vtable slot 2, fields via slot 4) whose column 3… |
| `00a82bc0` | `MatchmakingAvailableBuddiesDataSource::GetField` | ✅ | virtual | Lua/UI data-source field getter (vtable slot [4]): triggers the lazy rebuild (00a82590) when this+0x228 is clear, walks the buddy list from this+0x230 with stride 0x20 to row param_2 (count at this+0x238); column 0 copi… |
| `00a82c60` | `MatchmakingAvailableBuddiesDataSource::GetRowCount` | ✅ | virtual | Row-count getter (vtable slot [2]): if the loaded flag at this+0x228 is clear it first runs the rebuild routine 00a82590, then returns the buddy count stored at this+0x238. |
| `00a82e30` | `MatchmakingAvailableBuddiesDataSourceDtor` | ✅ | dtor | Destructor body of the Matchmaking.AvailableBuddies data source: stores the three class vtables, releases the two observer objects at param_1[0x9b] and param_1[0x9c] through their vtable slot [1] (delete with flag 1), s… |
| `00a831a0` | `MatchmakingAvailableBuddiesDataSourceCtor` | ✅ | ctor | Constructor of the Matchmaking.AvailableBuddies data source: registers the name "BaseClient.Matchmaking.AvailableBuddies" through 00922890, stores the three class vtables, saves the owner pointer at param_1[0x95], clear… |
| `00a832a0` | `MatchmakingAvailableBuddiesDataSourceScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot [0]) of MatchmakingAvailableBuddiesDataSource: runs the destructor body 00a82e30 and frees the object with _free when the delete flag bit 0 is set. |
