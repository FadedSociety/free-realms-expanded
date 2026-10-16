# Class: SoeUtil::Array<PlayerUpdatePacketRemoveNotifications::Entry,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bcea0` | `AllocateStorage` | ✅ | virtual | Adjusts requested capacity using the existing capacity field +0x0c and growth policy, then reuses storage or allocates param_2 entries at 16 bytes each with eight-byte alignment. |
| `008bcf40` | `FreeStorage` | ✅ | virtual | Calls FUN_00401c00 with the supplied pointer and alignment 8. |
| `008bcf60` | `ArrayPlayerUpdateRemoveNotificationsEntry::DispatchOrSwap` | ✅ | virtual | When the flag is nonzero, calls the supplied entry virtual slot at +0xc; otherwise exchanges its words at +4, +8 and +0xc with the destination. |
| `008c7450` | `SoeUtil_ArrayPlayerUpdateRemoveEntry_Dtor` | ✅ | dtor | Stores SoeUtil::Array<PlayerUpdatePacketRemoveNotifications::Entry,0,1>::vftable at this+0, zeroes the count at this+8, frees the buffer at this+4 through FUN_00401c00 with alignment 8, and zeroes this+4. Returns void. |
| `008c9230` | `SoeUtil_ArrayPlayerUpdateRemoveEntry_ScalarDeletingDtor` | ✅ | dtor | Stores the Array<PlayerUpdatePacketRemoveNotifications::Entry,0,1> vtable at this+0, zeroes the count at +8, frees the buffer at +4 through FUN_00401c00 with alignment 8, zeroes +4, and frees this with _free when bit 0 … |
