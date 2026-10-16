# Class: SoeUtil::HashListMap<int,ClientLobby::LobbyMember,100,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00baff60` | `HashListMap_ClientLobby_LobbyMember_100_Ctor` | ✅ | ctor | Constructor: stores the HashListMap<int,ClientLobby::LobbyMember,100,-1> vtable (01824f58), zeroes the dwords at +4, +8 and +0xc, and clears the 400-byte bucket array at +0x14 with memset. |
| `00bb0170` | `HashListMap_LobbyMember100_AllocNode` | 🟢 | virtual | Virtual slot [1] of the LobbyMember map: calls 006fb9c0, then allocates through the memory-manager chain (006fb860 virtual slot 1, else the global object DAT_01bdf280 virtual slot 1, else 006fb950 with size 0x18 and ali… |
| `00bb01b0` | `HashListMap_LobbyMember100_FreeNode` | ✅ | virtual | Virtual slot [2] of the LobbyMember map: calls 00401c00 (FreeMemory) on the node with alignment 8. |
| `00bb0610` | `HashListMap_LobbyMember100_DtorBody` | ✅ | dtor | Destructor body: stores the HashListMap<int,ClientLobby::LobbyMember,100,-1> vtable (01824f58) and calls 00bb0550 to clear all nodes. |
| `00bb0620` | `HashListMap_LobbyMember100_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot [0]): stores the HashListMap vtable (01824f58), calls 00bb0550 to clear the nodes, and calls _free when the delete flag bit 0 is set. |
| `00bb0730` | `HashListMap_LobbyMember100_DtorBody` | ✅ | dtor | Destructor body (second copy): stores the HashListMap vtable (01824f58) and calls 00bb0550 to clear all nodes. |
