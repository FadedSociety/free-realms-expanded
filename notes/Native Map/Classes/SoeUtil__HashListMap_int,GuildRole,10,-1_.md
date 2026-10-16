# Class: SoeUtil::HashListMap<int,GuildRole,10,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b1a160` | `HashListMap_GuildRole_Insert` | 🟢 | method | Inserts entry param_2 into the 10-bucket map: links it at the tail of the doubly-linked list at +0x18/+0x1c updating the head at +8 and tail at +0xc, pushes it onto the bucket chain (bucket = key at +0x20 mod 10, chain … |
| `00b1a730` | `HashListMap_GuildRole_Destruct` | 🟢 | dtor | Destructor body of the 10-bucket role map: for the head entry at +8 it unlinks the entry from its bucket chain (bucket = key at +0x20 mod 10, chain pointer at +0x24, bucket array at +0x14), repairs the doubly-linked lis… |
| `00b1af20` | `— ⚠reject` | ⚪ | method | Allocates a new role entry through virtual slot 4 of the map, initializes it with 0106d8e0 (which writes 0,0,0,-1,0), stores the key dword from the argument at +0x20 and inserts it with 00b1a160; a failed allocation ins… |
