# System: ui.leaderboard

25 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (12)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b85e80` | `ClientLeaderBoardManager::RequestLeaderBoard` | ClientLeaderBoardManager | Log and issue the config-driven web leaderboard request, throttled by the 1000 ms timer value. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b86390` | `LeaderBoardEntry_ParseAttributes` | LeaderBoardResponse::EntryLine | Parse one LeaderBoardResponse::EntryLine row from the XML result document. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b865b0` | `ClientLeaderBoardManager::ClientLeaderBoardManager` | ClientLeaderBoardManager | Creates the in-game leaderboard manager and its entry data source. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b86720` | `LeaderBoardResponse_ParseXml` | LeaderBoardResponse | Fill the leaderboard response (entries plus total page count) from the web reply; called by ClientLeaderBoardManager::OnGetResult after it locates the 'result' node. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b86840` | `ClientLeaderBoardManager::ScalarDeletingDtor` | ClientLeaderBoardManager | Deleting destructor for the leaderboard manager. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b86880` | `ClientLeaderBoardManager::OnGetResult` | ClientLeaderBoardManager | Consume the web leaderboard reply and republish the parsed rows to the leaderboard data source. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00cdee80` | `LeaderBoardEntryDataSource::DestructorBody` | LeaderBoardEntryDataSource | Destroys the leaderboard entry data source and its GUI table member. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |
| `00cdeef0` | `LeaderBoardDataSource::DestructorBody` | LeaderBoardDataSource | Destroys the leaderboard data source and its owned embedded objects. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |
| `00cdefb0` | `LeaderBoardEntryDataSource::Constructor` | LeaderBoardEntryDataSource | Constructs the entries data source for BaseClient.LeaderBoard.Entries. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |
| `00cdf030` | `LeaderBoardEntryDataSource::ScalarDeletingDestructor` | LeaderBoardEntryDataSource | MSVC scalar deleting destructor for LeaderBoardEntryDataSource. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |
| `00cdf050` | `LeaderBoardDataSource::Constructor` | LeaderBoardDataSource | Constructs the leaderboard data source and its entries data source. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |
| `00cdf110` | `LeaderBoardDataSource::ScalarDeletingDestructor` | LeaderBoardDataSource | MSVC scalar deleting destructor for LeaderBoardDataSource. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |

## 🟢 likely (11)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b856f0` | `LeaderBoardEntry_Id` | LeaderBoardResponse::EntryLine | Column accessor for entry id field +0x5c; matches the attribute-to-offset mapping of 00b86390 (id) and is called by the LeaderBoardEntryDataSource column switch at 00cded50. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b85710` | `LeaderBoardEntry_Type` | LeaderBoardResponse::EntryLine | Column accessor for entry type field +0x64; the parser 00b86390 reads the xml attribute 'type' into this same offset. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b85730` | `LeaderBoardEntry_Score` | LeaderBoardResponse::EntryLine | Column accessor for entry score field +0x68; the parser 00b86390 reads the xml attribute 'score' into this same offset. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b85750` | `LeaderBoardEntry_Rank` | LeaderBoardResponse::EntryLine | Column accessor for entry rank field +0x60; the parser 00b86390 reads the xml attribute 'rank' into this same offset, and the UI column switch 00cded50 is called with column 0 for it. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b85770` | `LeaderBoardEntry_Guid` | LeaderBoardResponse::EntryLine | Reads the entry guid/key pair (fields +0 and +4, initialized to the DAT_01be9ef8/efc defaults by the same-range ctor 00b85ce0); the parser 00b86390 stores the xml attribute 'guid' into these two dwords. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b857c0` | `LeaderBoardEntry_NameField` | LeaderBoardResponse::EntryLine | Column 1 of the leaderboard data source (00cded50) which wraps this value with FUN_0083d110; offset +0xc is the field right after the fixed-string base at +8 of the entry, so this is the name/string field accessor estab… | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b85ce0` | `LeaderBoardEntryLine_Ctor` | LeaderBoardResponse::EntryLine | Constructs one leaderboard entry (LeaderBoardResponse::EntryLine): the field set matches the 0x70-byte array element type of SoeUtil::Array<LeaderBoardResponse::EntryLine,0,1> and the accessors 00b856f0/00b85710/00b8573… | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b85d50` | `ClientLeaderBoardManager::LoadLeaderBoardUrl ⚠downgrade` | ClientLeaderBoardManager | Load the in-game leaderboard base URL from client configuration when the ClientLeaderBoardManager is constructed (called from the manager ctor 00b865b0). | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b86040` | `LeaderBoardEntry_CopyCtor` | LeaderBoardResponse::EntryLine | Element copy used by the append path 00b86300 and the resize/move path 00b860d0 of the SoeUtil::Array<LeaderBoardResponse::EntryLine,0,1> container. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b864d0` | `LeaderBoardResponse_Ctor` | LeaderBoardResponse | Initializes the leaderboard response object that OnGetResult fills and the data source 00cded50 reads (count at +0x14, data at +0x10). | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b86540` | `LeaderBoardResponse_Destroy` | LeaderBoardResponse | Releases the parsed leaderboard entries when the manager (00b86670) is destroyed. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0090d5c0` | `FormatLeaderboardTimeEntry ?` |  | Formats a leaderboard time entry for display. | [[Native Map/Shards/S0239 0090a890-009105b0|S0239]] |

## ⚪ unknown (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b86670` | `— ⚠reject` | ClientLeaderBoardManager | Tears down the leaderboard manager. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |

