# Class: SoeUtil::List<SoeUtil::StrongType<unsigned___int64,SoeUtilSignatureCharacterGuid,0>,20>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a8c9f0` | `PoolEntryAlloc` | 🟢 | function | Pops the head entry of the free list at this+8: when the list is empty it first calls 00a8c870 (PoolChunkGrow) with 0; decrements the count at this+0xc, stores the next link into this+8, and when the flag byte at this+0… |
| `00a8ca40` | `PoolEntryFree` | 🟢 | function | Pushes the entry param_2 onto the free list at this+8, increments the count at this+0xc, and when the flag byte at this+0x10 is set finds the owning chunk through 00a8c350, increments its word at +0x18 and, when that re… |
| `00a8cb50` | `ListGuid20VSlot2Alloc ?` | 🟡 | virtual | Virtual slot 2 of SoeUtil::List<StrongType<uint64,SoeUtilSignatureCharacterGuid,0>,20>: adds 0x10 to ECX (83 c1 10) and jumps to 00a8c9f0 (PoolEntryAlloc, rel32 e9), so it is a this-adjusting jump thunk into the list bo… |
| `00a8cb60` | `ListGuid20VSlot3Free ?` | 🟡 | virtual | Virtual slot 3 (+0xc) of List<StrongType<uint64,SoeUtilSignatureCharacterGuid,0>,20>: bytes are add ecx,0x10 followed by a rel32 jump to 00a8ca40 (PoolEntryFree); it is a this-adjusting jump thunk. |
| `00a8cc70` | `ListGuid20ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor of List<StrongType<uint64,SoeUtilSignatureCharacterGuid,0>,20>: calls 00a8caa0 (ListGuid20DtorBody), then calls _free(this) through thunk 00d82463 when bit 0 of param_2 is set; returns this. |
