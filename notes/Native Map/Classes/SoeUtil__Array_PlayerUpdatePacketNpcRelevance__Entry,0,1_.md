# Class: SoeUtil::Array<PlayerUpdatePacketNpcRelevance::Entry,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bcfd0` | `NpcRelevanceEntryArray::Allocate` | ✅ | virtual | Computes a resized capacity from the current and requested counts, stores that count through param_3, and allocates count times 0x18 bytes aligned to 8; may reuse the current buffer. |
| `008bd070` | `NpcRelevanceEntryArray::FreeBuffer` | ✅ | virtual | Calls FreeMemory with the supplied pointer and 8-byte alignment, then returns. |
| `008bd090` | `ArrayPlayerUpdateNpcRelevanceEntry::DispatchOrSwap` | ✅ | virtual | When the flag is nonzero, calls the supplied entry virtual slot at +0xc; otherwise exchanges its words at +4, +8 and +0xc with the destination. |
| `008c7480` | `SoeUtil_ArrayPlayerUpdateNpcRelevance_Dtor` | ✅ | dtor | Stores SoeUtil::Array<PlayerUpdatePacketNpcRelevance::Entry,0,1>::vftable at this+0, zeroes the count at this+8, frees the buffer at this+4 through FUN_00401c00 with alignment 8, and zeroes this+4. Returns void. |
| `008c9270` | `SoeUtil_ArrayPlayerUpdateNpcRelevance_ScalarDeletingDtor` | ✅ | dtor | Stores the Array<PlayerUpdatePacketNpcRelevance::Entry,0,1> vtable at this+0, zeroes the count at +8, frees the buffer at +4 through FUN_00401c00 with alignment 8, zeroes +4, and frees this with _free when bit 0 of the … |
