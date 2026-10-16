# Class: SoeUtil::Array<FriendUpdatePositionsPacket::Entry,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c7e380` | `FriendPositionEntry::AllocateArray` | ✅ | function | Reuses storage under capacity rules or adjusts capacity, stores it, and allocates capacity times 0x28 bytes aligned to 8. |
| `00c7e420` | `FriendPositionEntry::FreeArray` | ✅ | function | Frees the supplied backing-storage pointer with alignment 8 via FUN_00401c00. |
| `00c7e440` | `FriendPositionEntrySwap` | ✅ | function | When move flag is set invokes an entry virtual operation; otherwise swaps pointer +4 and XOR-swaps 32-bit fields +8 and +0xc between entries. |
| `00c7ffb0` | `FriendUpdatePositionsEntryArrayDtor` | ✅ | dtor | Installs the array vtable, destroys array elements, frees backing storage with alignment 8, and clears its storage pointer. |
| `00c802f0` | `FriendUpdatePositionsEntryArrayScalarDeletingDtor` | ✅ | dtor | Destroys array elements, frees backing storage with alignment 8, and frees the array object when flag bit 0 is set. |
