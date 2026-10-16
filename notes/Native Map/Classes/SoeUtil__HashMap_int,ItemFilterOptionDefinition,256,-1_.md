# Class: SoeUtil::HashMap<int,ItemFilterOptionDefinition,256,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0107d540` | `HashMapSlot1AllocNode0x20` | 🟢 | virtual | Virtual slot 1: allocates 0x20 bytes, alignment 4, through the thread memory manager (vslot1), else the global manager DAT_01bdf280 (vslot1), else calls 006fb950(0x20, 4). Same shape as 0107d450 with size 0x20. |
| `0107d580` | `HashMapSlot2FreeNode` | 🟢 | virtual | Virtual slot 2, stdcall-style (ret 4): pushes alignment 4 and the pointer argument, calls 00401c00 (FreeMemory). Same shape as 0107d490. |
| `0107ddb0` | `— ⚠reject` | ⚪ | method | Removes node base (param_2-4) from a 256-bucket hash: bucket index is the byte at node-4 masked 0xff into the bucket array at this+0xc, chain link is node+0x1c; decrements count this+0x8; calls FUN_0108c4c0 (empty stub)… |
| `0107de40` | `HashMapClearFreeAllNodes ?` | 🟡 | method | Walks all 256 bucket heads at this+0xc, zeroes each bucket, and for every chained node calls the empty stub 0108c4c0, then vslot 2 (free) with the node, decrementing count this+0x8 per node. Stops early when the count r… |
| `0107e010` | `HashMapDtorBody` | 🟢 | dtor | Non-deleting destructor body (bytes: mov [ecx],vftable; jmp 0107de40): stores the HashMap<int,ItemFilterOptionDefinition,256,-1> vftable into this, then tail-jumps to 0107de40 (clears all buckets, freeing nodes). Called… |
| `0107e020` | `HashMapScalarDeletingDtor` | 🟢 | dtor | Virtual slot 0 scalar deleting destructor: stores the vftable, calls 0107de40 (clear), then if flag bit 0 of the argument is set calls _free(this), returns this. |
| `0107e180` | `— ⚠reject` | ⚪ | method | Allocates a node through vslot 1 (0x20-byte node); if the allocation is non-null, constructs it with 0107d2a0 (key dword from param_2 into node+0, value from param_3 via FUN_0108c4a0), links it at the head of bucket (ke… |
