# Class: LeaderBoardResponse

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b864d0` | `LeaderBoardResponse_Ctor` | 🟢 | ctor | Constructor of the response wrapper that owns the entry array at +0xc: puts the SoeUtil::Array<LeaderBoardResponse::EntryLine,0,1> vtable at this+0xc, zeroes this+0x10/+0x14/+0x18 (array data/count/capacity) and this+0/… |
| `00b86540` | `LeaderBoardResponse_Destroy` | 🟢 | dtor | Destructor body of the response wrapper: resets the entry-array vtable at this+0xc, destroys the elements counted at this+0x14 through 00b86200, frees the element buffer at this+0x10 with alignment 8 via FUN_00401c00 an… |
| `00b86720` | `LeaderBoardResponse_ParseXml` | ✅ | method | Parses a leaderboard XML result document into a response object: reads the 'totalPages' attribute into this+0x1c, constructs a temporary entry (00b85ce0), then while the DOM cursor advances to an 'entry' node it parses … |
