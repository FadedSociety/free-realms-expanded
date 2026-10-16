# Class: SoeUtil::HashListMap<int,MountClientInfo,16,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b03cd0` | `HashListMap_MountClientInfo_Ctor ⚠downgrade` | 🟢 | ctor | Installs SoeUtil::HashListMap<int,MountClientInfo,16,-1>::vftable at +0, zeroes the header dwords at +4, +8, +0xc, +0x10 (size, head, tail, count) and memsets 0x40 bytes (16 hash buckets) at +0x14; returns the object. |
| `00b04350` | `HashListMap_MountClientInfo_AllocNode` | ✅ | virtual | Node allocation hook, vtable slot 1 of both the HashListMap and its HashList base: calls FUN_006fb9c0, then FUN_006fb860; when that returns a manager it calls the manager's virtual slot 1 and returns, otherwise it retri… |
| `00b04390` | `HashListMap_MountClientInfo_FreeNode` | ✅ | virtual | Node deallocation hook, vtable slot 2: routes the node pointer and the alignment 8 to the memory manager free path FUN_00401c00(pointer,8) and returns. |
| `00b057c0` | `HashListMapDestructor` | 🟢 | dtor | Sets the HashListMap vtable and repeatedly removes the current node until the map is empty. |
| `00b057f0` | `ScalarDeletingDtor ? → SoeUtil::HashListMap<int,MountClientInfo,16,-1>::ScalarD…` | 🟡 | dtor | Sets the hash-map vtable, removes all nodes, returns this when delete flag is clear, and frees this when set. |
| `00b058a0` | `HashListMapDestructor` | 🟢 | dtor | Installs the HashListMap vtable and removes nodes until its map is empty. |
| `00b059c0` | `MountInfoMapConstructor` | 🟢 | ctor | Installs HashListMap then HashList vtables, clears links and 16 buckets, and returns this. |
