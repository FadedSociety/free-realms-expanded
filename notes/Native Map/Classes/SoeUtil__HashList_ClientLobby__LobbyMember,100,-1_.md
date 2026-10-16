# Class: SoeUtil::HashList<ClientLobby::LobbyMember,100,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bb07f0` | `HashList_ClientLobby_LobbyMember_100_Ctor` | ✅ | ctor | Constructor: stores the HashListMap<...LobbyMember,100,-1> vtable (01824f58), zeroes the dwords at +4, +8 and +0xc, clears the 400-byte bucket array at +0x14 with memset, then stores the HashList<ClientLobby::LobbyMembe… |
| `00bb0870` | `HashList_LobbyMember100_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot [0]): stores the HashListMap vtable (01824f58), calls 00bb0550 to clear the nodes, and calls _free when the delete flag bit 0 is set. |
