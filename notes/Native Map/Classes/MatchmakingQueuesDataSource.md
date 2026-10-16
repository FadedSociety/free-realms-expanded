# Class: MatchmakingQueuesDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a7f2f0` | `MatchmakingQueuesDataSource::GetColumnName` | ✅ | virtual | Column-name table of the matchmaking queue list: QueueId, Name, MatchType, MinPlayers, MaxPlayers, MinTeams, MaxTeams, MaxGameStartDelay, Param1..Param7, PendingRequests, AverageWait, EncounterDescription, EncounterIcon… |
| `00a7fe00` | `MatchmakingQueuesDataSource::GetRowCount` | 🟢 | virtual | Lazily builds the rows once: when the byte at this+0x228 is clear it calls virtual slot 3 (offset 0xc) on the sink object at this+4 and sets that byte to 1, then returns the row count stored at this+0x238. |
| `00a81e90` | `MatchmakingQueuesDataSource::GetField` | ✅ | virtual | Lua/UI data-source field getter (vtable slot [4]): lazily loads rows (calls the loader through this+4 slot 3 once, setting flag this+0x228), walks the row list from this+0x230 (stride 0x54, count at this+0x238) to row p… |
| `00a82d50` | `MatchmakingQueuesDataSourceDtor` | ✅ | dtor | Destructor body of the Matchmaking.Queues data source: stores the three MatchmakingQueuesDataSource vtables (main object plus two sub-objects at +0x30 and +0x34), destroys the two SoeUtil::List<int,-1> members at param_… |
| `00a830a0` | `— ⚠reject` | ⚪ | ctor | Constructor of the Matchmaking.Queues data source: registers the UI data source name through 00922890(PTR_s_BaseClient.Matchmaking.Queues), stores the three class vtables, saves the owner pointer at param_1[0x95], clear… |
| `00a83180` | `MatchmakingQueuesDataSourceScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot [0]) of MatchmakingQueuesDataSource: runs the destructor body 00a82d50 and frees the object with _free when the delete flag bit 0 is set. |
| `00a83c00` | `— ⚠reject` | ⚪ | handler | Reacts to a matchmaking server status change (param_2 non-zero = up): when up it first constructs a ListQueuesRequestPacket (00a7fb90), reads the client's current connection handle pair from client+0x3a0/+0x3a4, sends t… |
