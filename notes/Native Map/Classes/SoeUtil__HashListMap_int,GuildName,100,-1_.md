# Class: SoeUtil::HashListMap<int,GuildName,100,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a1be20` | `GuildNameMap_Dtor` | 🟢 | dtor | Installs the GuildName map vftable and repeatedly removes the head node through 00a19c60 until the list is empty. |
| `00a1be50` | `GuildNameMap_AllocNode ?` | 🟡 | virtual | Returns storage from the current thread allocator when available, otherwise the global allocator, otherwise the 0x50-byte fallback allocation aligned to eight. |
| `00a1be90` | `GuildNameMap_FreeNode` | 🟢 | virtual | Pushes size eight and the supplied pointer, calls allocator free 00401c00, adjusts the stack, and returns. |
| `00a1bee0` | `GuildNameMap_ScalarDeletingDtor` | 🟢 | dtor | Destroys every GuildName map node through 00a19c60, conditionally frees this object, and otherwise returns it. |
| `00a1c0f0` | `GuildNameMap_DtorBody` | 🟢 | dtor | Installs the GuildName map vftable, repeatedly removes nodes through 00a19c60, and restores ExceptionList. |
| `00b177b0` | `HashListMap_GuildName_Construct` | ✅ | ctor | Constructor of the guild name map: installs the HashListMap vtable, zeroes the list head, tail count and the field at +4, and memsets the 100 inline bucket slots at +0x14 to zero (400 bytes). |
| `00b17b90` | `HashListMap_GuildName_Remove` | 🟢 | method | Removes one entry from the 100-bucket guild name map: takes the key at entry+0xe0, computes the bucket as key % 100, walks the chain rooted at this+0x14+bucket*4 following the next pointers at entry+0xe4 and unlinks the… |
| `00b19170` | `HashListMap_GuildName_Erase` | 🟢 | method | Erase-and-delete for the 100-bucket guild name map: unlinks the entry with 00b17b90, repairs the doubly-linked list at entry+0xd8/entry+0xdc while updating the map head and tail at +8/+0xc, decrements the count at +0x10… |
| `00b1a100` | `HashListMap_GuildName_Insert` | 🟢 | method | Inserts entry param_2 into the 100-bucket guild name map: links it at the tail of the doubly-linked list at +0xd8/+0xdc updating the head at +8 and the tail at +0xc, pushes it onto the bucket chain (bucket = key at +0xe… |
| `00b1a710` | `HashListMap_GuildName_Destruct` | 🟢 | dtor | Destructor body of the guild name map: repeatedly takes the list head at +8 and passes it to 00b19170, which unlinks the entry from the bucket chain, repairs the doubly-linked list and frees it, until the head is null; … |
| `00b1aea0` | `— ⚠reject` | ⚪ | method | Allocates a new guild name entry through virtual slot 4 of the map, constructs it with 00b17d20 (which sets the key at +0xe0 from the argument) and inserts it with 00b1a100; a failed allocation inserts null. |
| `00b1e2b0` | `GuildNameHashListMapCtor` | ✅ | ctor | Installs the HashListMap and HashList vtables, zeros the map state, clears 400 bytes for 100 bucket heads, and returns the map pointer. |
