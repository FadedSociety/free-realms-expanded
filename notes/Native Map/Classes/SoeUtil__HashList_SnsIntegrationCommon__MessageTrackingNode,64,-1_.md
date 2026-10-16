# Class: SoeUtil::HashList<SnsIntegrationCommon::MessageTrackingNode,64,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0107fcc0` | `HashListSnsSlot1AllocNode` | 🟢 | virtual | Virtual slot 1: calls 006fb9c0 once, then 006fb860 for the thread memory manager; if present calls its vslot 1; else the global manager DAT_01bdf280 vslot 1; else calls 006fb950(0x58, 8). Allocates a 0x58-byte, 8-aligne… |
| `0107fd00` | `HashListSnsSlot2FreeNode → HashListAndHashListMapSnsSlot2FreeAligned8` | 🟢 | virtual | Virtual slot 2, stdcall-style (ret 4): pushes alignment 8 and the pointer, calls 00401c00 (FreeMemory). Bytes decoded. |
| `01081d70` | `HashListSnsCtor` | 🟢 | ctor | Constructor body: stores the HashListMap<int,SnsIntegrationCommon::MessageTrackingNode,64,-1> vftable, zeroes this+0x10, +0xc, +0x8 and +0x4, clears the 64-bucket array at this+0x14 (_memset 0x100 bytes), then stores th… |
| `01081df0` | `HashListSnsScalarDeletingDtor` | 🟢 | dtor | Virtual slot 0 scalar deleting destructor: calls the destructor body 01081bf0 on this, then frees this with _free when flag bit 0 of the argument is set, and returns this. |
