# Class: SoeUtil::HashList<ClientActivityDefinition::FeaturedActivityEntry,16,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01082570` | `HashListActivitySlot1AllocNode` | 🟢 | virtual | Virtual slot 1: calls 006fb9c0 once, then 006fb860 for the thread manager; if present calls its vslot 1; else the global manager vslot 1; else calls 006fb950(0x30, 8). Allocates a 0x30-byte, 8-aligned node. |
| `010825b0` | `HashListActivitySlot2FreeNode` | 🟢 | virtual | Virtual slot 2, stdcall-style (ret 4): pushes alignment 8 and the pointer, calls 00401c00 (FreeMemory). Bytes not decoded beyond the pseudocode; the 18-byte size matches the stdcall free wrapper pattern. |
| `01082920` | `HashListActivityCtor16` | 🟢 | ctor | Constructor body: stores the HashListMap<int,ClientActivityDefinition::FeaturedActivityEntry,16,-1> vftable, zeroes +0x10, +0x8, +0xc and +0x4, clears the 16-bucket array at this+0x14 (_memset 0x40 bytes), then stores t… |
| `010829a0` | `HashListActivityScalarDeletingDtor` | 🟢 | dtor | Virtual slot 0 scalar deleting destructor: calls the destructor body 010827f0 on this, then frees this with _free when flag bit 0 of the argument is set, and returns this. |
