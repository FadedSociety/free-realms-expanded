# Class: SoeUtil::HashMap<int,ClientGuildManager::PendingGuildNameRequest,32,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a13660` | `HashMap_PendingGuildNameRequest_ScalarDeletingDtor` | 🟢 | dtor | Scalar deleting destructor (vtable slot 0) of the guild manager's pending name-request map: writes the class vftable into the object, runs the 32-bucket teardown loop FUN_00a11090 (walks the bucket array at this+0xc, fr… |
| `00a13ed0` | `HashMap_PendingGuildNameRequest_DtorWithEH` | ✅ | dtor | Installs the PendingGuildNameRequest map vftable and calls FUN_00a11090 with this object in ECX to drain the 32-bucket map, then returns without freeing the object. |
| `00a1cea0` | `PendingGuildNameRequestMap_Dtor` | 🟢 | dtor | Releases two guarded references, destroys the GuildName hash-list member, installs the 32-bucket map vftable, and clears its nodes. |
| `00b17690` | `HashMap_PendingGuildNameRequest_Construct` | ✅ | ctor | Constructor of the pending guild name request map: installs the HashMap vtable, zeroes the entry count at +8, memsets the 32 inline bucket slots at +0xc to zero (0x80 bytes) and zeroes the field at +4. |
