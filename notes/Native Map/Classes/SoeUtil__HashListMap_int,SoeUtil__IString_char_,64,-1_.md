# Class: SoeUtil::HashListMap<int,SoeUtil::IString<char>,64,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00976b90` | `SoeUtil::HashListMapIntIString64Ctor` | ✅ | ctor | Stores the HashListMap<int,IString<char>,64,-1> vtable at this+0, zeroes the fields at +4, +8, +0xc and +0x10, memsets the 0x100-byte bucket table at this+0x14 (64 pointers), and returns this. |
| `009771f0` | `SoeUtil::HashListMapIntIString64AllocNode` | ✅ | virtual | Virtual slot [1] of the map. Calls FUN_006fb9c0, then takes the allocator returned by FUN_006fb860; if that is non-null it calls its vtable slot +4 with (0x20, 4). Otherwise, if DAT_01bdf280 is non-null, it calls that o… |
| `00977230` | `SoeUtil::HashListMapIntIString64FreeNode` | ✅ | virtual | Virtual slot [2] of the map: calls FUN_00401c00(block, 4), the free routine with alignment 4, and returns (ret 4). |
| `00977ba0` | `SoeUtil::HashListMapIntIString64RemoveNode` | 🟢 | method | Removes node param_2 from this container: selects bucket (key at node+0x18 & 0x3f) at this+0x14, unlinks the node from that bucket chain (next at node+0x1c), unlinks it from the doubly linked list through node+0x10/+0x1… |
| `00978680` | `— ⚠reject` | ⚪ | method | Allocates a node through the container's vtable slot +4 (returns without linking when that is 0), constructs it with FUN_00977cc0(param_2, param_3), links it at the tail of the list (old tail stored at node+0x14, next a… |
| `00978760` | `SoeUtil::HashListMapIntIString64Clear` | 🟢 | method | Reads the list head at this+8 and repeatedly removes it with 00977ba0 until the head is null. Returns nothing. |
| `00978df0` | `SoeUtil::HashListMapIntIString64DtorBody` | ✅ | dtor | Stores the HashListMap<int,IString<char>,64,-1> vtable at this+0 and then removes nodes from the list head through 00977ba0 until the head at this+8 is null. Returns nothing. |
| `00978f10` | `SoeUtil::HashListMapIntIString64DtorBodyB` | ✅ | dtor | Stores the HashListMap vtable at this+0 and removes nodes from the list head through 00977ba0 until this+8 is null. Same body as 00978df0. Returns nothing. |
