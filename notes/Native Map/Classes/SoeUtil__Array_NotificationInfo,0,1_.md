# Class: SoeUtil::Array<NotificationInfo,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bcd70` | `AllocateStorage → ResizeStorageWithGrowthPolicy` | ✅ | virtual | Adjusts requested capacity using the existing capacity field +0x0c and growth policy, then reuses storage or allocates param_2 entries at 0x30 bytes each with eight-byte alignment. |
| `008bce10` | `FreeStorage` | ✅ | virtual | Calls FUN_00401c00 with the supplied pointer and alignment 8. |
| `008bce30` | `SwapOrDestroyStorage ? ⚠rename` | 🟡 | virtual | If the destroy flag is set, calls param_2's virtual +0x0c; otherwise swaps storage pointer, capacity and size fields between the arrays. |
| `008c7420` | `SoeUtil_ArrayNotificationInfo_Dtor` | ✅ | dtor | Stores SoeUtil::Array<NotificationInfo,0,1>::vftable at this+0, zeroes the count at this+8, frees the buffer at this+4 through FUN_00401c00 with alignment 8 (push 8 in the bytes), and zeroes this+4. Returns void. |
| `008c91f0` | `SoeUtil_ArrayNotificationInfo_ScalarDeletingDtor` | ✅ | dtor | Stores the Array<NotificationInfo,0,1> vtable at this+0, zeroes the count at +8, frees the buffer at +4 through FUN_00401c00 with alignment 8, zeroes +4, and frees this with _free when bit 0 of the flag byte is set. Ret… |
