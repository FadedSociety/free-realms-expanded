# Class: SoeUtil::List<Fotomat::NpcEntry,4096>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bd560` | `FotomatNpcEntryList4096::InitStorage` | 🟢 | init | Initializes list state and 0x400 embedded nodes; its int pointer advances by 0x70 elements (0x1c0 bytes) per iteration, then stores capacity 0x1000. |
| `009188e0` | `— ⚠reject` | ⚪ | ctor | Stores the List<Fotomat::NpcEntry,-1> vtable, zeroes the fields at +4, +8 and +0xc, then stores the List<Fotomat::NpcEntry,4096> vtable as the final one. Then calls FUN_008bd560 with no visible effect on this object. Re… |
| `009190b0` | `— ⚠reject` | ⚪ | method | Thiscall: computes n = min(argument, clamp(FUN_00918b20 result, 2, 0x924)), allocates n*0x70 + 0x20 bytes with alignment 0x10 through the allocator (FUN_006fb950, or the object at DAT_01bdf280 when set), inserts the chu… |
| `009191d0` | `SoeUtil::ListNpcEntryReleaseChunk` | 🟢 | method | Thiscall with a chunk header (param_2). Walks the free list at this+8 and unlinks every element whose address lies inside the chunk (chunk start at header dword 0, end at start + count*0x70). Subtracts the count from th… |
| `00919230` | `SoeUtil::ListNpcEntryPopFreeNode` | 🟢 | virtual | Reached through the vtable thunk 00919310 (this += 0x10). If the free list at this+8 is empty it calls FUN_009190b0 (grow). Takes the head node, sets this+8 to its first dword, decrements the count at this+0xc, and when… |
| `009192a0` | `SoeUtil::ListNpcEntryPushFreeNode` | 🟢 | virtual | Reached through the vtable thunk 00919320 (this += 0x10). Stores the old free-list head into the node's first dword, increments this+0xc, sets this+8 to the node. When the byte at this+0x10 is set it finds the owning ch… |
| `00919310` | `ThunkAdjustThis10_ListNpcEntry4096 ?` | 🟡 | virtual | Decoded from bytes: add ecx,0x10 then jmp rel32 to 00919230. A this-adjusting thunk with no body of its own. |
| `00919320` | `ListNpcEntry4096Vftable3ThisAdjustingThunkTo009192a0` | ✅ | virtual | This-adjusting thunk: adds 0x10 to ECX, then jumps to 009192a0. Its vtable slot is [3]. |
| `00919330` | `SoeUtil::ListNpcEntry4096DtorBody` | ✅ | dtor | Destructor body: stores the List<Fotomat::NpcEntry,4096> vtable, walks and frees the node chain at +4 via FUN_008e47b0, then if the member at +0x10 is non-null frees its two tree children with FUN_008cb0f0(.,0x10) and t… |
| `00919440` | `SoeUtil::ListNpcEntry4096DeletingDtor` | ✅ | dtor | Scalar deleting destructor, thiscall with flag byte: calls the destructor body 00919330, then when flag bit 0 is set frees the object with _free. Returns the object pointer. |
