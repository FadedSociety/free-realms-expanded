# Class: SoeUtil::List<GameAccountManagement::ServerDetails,10>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00713010` | `ListServerDetails10_Ctor` | ✅ | ctor | Stores SoeUtil::List<GameAccountManagement::ServerDetails,-1>::vftable at [this], zeroes [this+4], [this+8] and [this+0xc], stores SoeUtil::List<GameAccountManagement::ServerDetails,10>::vftable at [this], calls 007088a… |
| `00713a40` | `ThunkAddThis10_00713960` | ✅ | virtual | Thunk: adds 0x10 to ECX (83 c1 10) and jumps to 00713960 (e9 18 ff ff ff). |
| `00713a50` | `ThunkAddThis10_007139d0` | ✅ | virtual | Thunk: adds 0x10 to ECX (83 c1 10) and jumps to 007139d0 (e9 78 ff ff ff). |
| `00713a60` | `ListServerDetails10_DtorBody` | ✅ | dtor | Stores SoeUtil::List<GameAccountManagement::ServerDetails,10>::vftable at [this]; unlinks and releases the list nodes at [this+4] through 0070e5d0 until empty; zeroes [this+0x18] and [this+0x1c]; frees the block tree ro… |
| `00713b50` | `ListServerDetails10_ScalarDtor` | ✅ | dtor | Calls 00713a60 (list destructor body) with ECX=this, then calls _free(this) when bit 0 of the stack byte is set; returns this. |
