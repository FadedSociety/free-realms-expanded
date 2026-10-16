# Class: SoeUtil::List<GameAccountManagement::FriendDetails,100>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `007124f0` | `ListFriendDetails100_Ctor` | ✅ | ctor | Stores SoeUtil::List<GameAccountManagement::FriendDetails,-1>::vftable at [this], zeroes [this+4], [this+8] and [this+0xc], stores SoeUtil::List<GameAccountManagement::FriendDetails,100>::vftable at [this], calls 007087… |
| `00712f20` | `ThunkAddThis10_00712e40` | ✅ | virtual | Thunk: adds 0x10 to ECX (83 c1 10) and jumps to 00712e40 (e9 18 ff ff ff). |
| `00712f30` | `ThunkAddThis10_00712eb0` | ✅ | virtual | Thunk: adds 0x10 to ECX (83 c1 10) and jumps to 00712eb0 (e9 78 ff ff ff). |
| `00712f40` | `ListFriendDetails100_DtorBody` | ✅ | dtor | Stores SoeUtil::List<GameAccountManagement::FriendDetails,100>::vftable at [this]; unlinks and releases the list nodes at [this+4] through 0070e530 until empty; zeroes [this+0x18] and [this+0x1c]; frees the block tree r… |
| `00713b30` | `ListFriendDetails100_ScalarDtor` | ✅ | dtor | Calls 00712f40 (list destructor body) with ECX=this, then calls _free(this) when bit 0 of the stack byte is set; returns this. |
