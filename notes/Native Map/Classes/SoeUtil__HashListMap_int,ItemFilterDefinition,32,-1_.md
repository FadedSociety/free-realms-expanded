# Class: SoeUtil::HashListMap<int,ItemFilterDefinition,32,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0107d450` | `HashListMapSlot1AllocNode` | 🟢 | virtual | Virtual slot 1, no stack args from the caller. Bytes: calls 006fb9c0 once, then 006fb860 for the current thread memory manager; if present calls its vslot1 with pushed (0x138, 4); else the global manager DAT_01bdf280 vs… |
| `0107d490` | `HashListMapSlot2FreeNode` | 🟢 | virtual | Virtual slot 2, stdcall-style (ret 4): pushes alignment 4 and the pointer argument, calls 00401c00 (FreeMemory, which routes to the thread manager, global manager or CRT free). Bytes decoded. |
| `0107dce0` | `— ⚠reject` | ⚪ | method | Removes node param_2 from this container: bucket index is (node+0x130 & 0x1f) into the bucket array at this+0x14, chain link is node+0x134; unlinks node from the global list (next +0x128, prev +0x12c, head this+0x8, tai… |
| `0107dff0` | `HashListMapClearViaRemove ?` | 🟡 | method | Loop: while the list head at this+0x8 is non-null, calls 0107dce0 (HashListMapRemoveNode) on the head. Empties the list through the removal helper. |
| `0107e110` | `HashListMapDtorBody` | 🟢 | dtor | Non-deleting destructor body: stores the HashListMap<int,ItemFilterDefinition,32,-1> vftable into this, then removes nodes from the list head (this+0x8) through 0107dce0 until the head is null. Called from two EH unwind… |
| `0107e210` | `HashListMapScalarDeletingDtor → SoeUtil::HashListMap<int,ItemFilterDefinition,3…` | 🟢 | dtor | Virtual slot 0 scalar deleting destructor: stores the vftable, removes nodes from the list head through 0107dce0 until the head is null, then frees this when flag bit 0 of the argument is set and returns this. |
| `0107f060` | `— ⚠reject` | ⚪ | method | Thiscall: allocates a node through vslot 1 of this (null returns), constructs it with 0107d1f0 (key dword at node+0x130), appends it at the list tail (successor +0x128, predecessor +0x12c, head this+0x8, tail this+0xc),… |
