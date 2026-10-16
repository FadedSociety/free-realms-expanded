# Class: SoeUtil::HashList<CharacterStatDefinition,71,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0102b360` | `HashListCharStatAllocSlot` | ✅ | virtual | Virtual allocate slot [1]: calls the lock helper 006fb9c0, then allocates through the thread-manager entry 006fb860 and its vslot +4, else DAT_01bdf280 vslot +4, else 006fb950 with size 0x20 and alignment 4. |
| `0102b3a0` | `HashListCharStatFreeSlot` | ✅ | virtual | Virtual free slot [2]: calls 00401c00 (FreeMemory) with the entry pointer and alignment 4, then returns. |
| `0102bb60` | `HashList::DeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores the HashListMap<int,CharacterStatDefinition,71,-1> vftable, calls the destructor body 0102b610, then calls _free on this when bit 0 of the flag argument is set; returns this. |
