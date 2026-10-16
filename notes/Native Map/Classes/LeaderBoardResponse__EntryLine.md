# Class: LeaderBoardResponse::EntryLine

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b856f0` | `LeaderBoardEntry_Id` | 🟢 | method | Bounds-checked element getter on a 0x70-byte-stride array: returns 0 when index param_2 is negative or >= the count at this+0x14, otherwise returns the dword at element*0x70 + 0x5c using the base pointer at this+0x10. |
| `00b85710` | `LeaderBoardEntry_Type` | 🟢 | method | Bounds-checked element getter on a 0x70-byte-stride array: returns 0 when index param_2 is negative or >= this+0x14, otherwise the dword at element*0x70 + 0x64 from the base at this+0x10. |
| `00b85730` | `LeaderBoardEntry_Score` | 🟢 | method | Bounds-checked element getter on a 0x70-byte-stride array: returns 0 when index param_2 is out of range for the count at this+0x14, otherwise the dword at element*0x70 + 0x68 from the base at this+0x10. |
| `00b85750` | `LeaderBoardEntry_Rank` | 🟢 | method | Bounds-checked element getter on a 0x70-byte-stride array: returns 0 when index param_2 is out of range for the count at this+0x14, otherwise the dword at element*0x70 + 0x60 from the base at this+0x10. |
| `00b85770` | `LeaderBoardEntry_Guid` | 🟢 | method | Bounds-checked 8-byte element getter: when index param_3 is in range it copies the two dwords at element*0x70 + 0 and +4 from the base at this+0x10 into the out pair param_2; when out of range it writes the default cons… |
| `00b857c0` | `LeaderBoardEntry_NameField` | 🟢 | method | Bounds-checked element getter returning the dword at element*0x70 + 0xc (base this+0x10, count this+0x14), or 0 when the index is out of range. |
| `00b85ce0` | `LeaderBoardEntryLine_Ctor` | 🟢 | ctor | Constructor for a 0x70-byte entry record: runs StringFixed64Ctor (FUN_00402a10) on this+8 (the entry name string member), zeroes the dwords at +0x5c, +0x60, +0x64 and +0x68, and initializes the 8-byte key at +0/+4 from … |
| `00b86040` | `LeaderBoardEntry_CopyCtor` | 🟢 | method | Copy-constructs a 0x70-byte entry into param_1 from param_2: copies the 8-byte key at +0/+4, copies the fixed-string member at +8 through FUN_00964840, and copies the four scalar fields at +0x5c, +0x60, +0x64 and +0x68 … |
| `00b86390` | `LeaderBoardEntry_ParseAttributes` | ✅ | method | Populates one entry record from a DOM reader: reads the 64-bit 'guid' attribute into the entry key at +0 (00b85b20), then the named ints 'rank' into +0x60, 'id' into +0x5c, 'score' into +0x68 and 'type' into +0x64 throu… |
