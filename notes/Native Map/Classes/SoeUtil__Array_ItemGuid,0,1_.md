# Class: SoeUtil::Array<ItemGuid,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bbe40` | `AllocateStorage` | ✅ | virtual | Uses the existing capacity at +0x0c in its growth/reuse decision, writes the selected capacity through param_3, returns the existing pointer at +4 when reused, or allocates param_2 four-byte ItemGuid elements aligned to… |
| `008bbee0` | `FreeStorage` | ✅ | virtual | Calls FUN_00401c00 with the supplied pointer and alignment 4. |
| `008bbf00` | `SwapOrDestroyStorage` | ✅ | virtual | If the destroy flag is set, calls param_2's virtual +0x0c; otherwise swaps storage pointer, capacity and size fields between the arrays. |
| `008c6f60` | `SoeUtil_ArrayItemGuid_Dtor` | ✅ | dtor | Stores SoeUtil::Array<ItemGuid,0,1>::vftable at this+0, zeroes the count at this+8, frees the buffer at this+4 through FUN_00401c00 with alignment 4, and zeroes this+4. Returns void. |
| `008c9170` | `SoeUtil_ArrayItemGuid_ScalarDeletingDtor` | ✅ | dtor | Stores the Array<ItemGuid,0,1> vtable at this+0, zeroes the count at +8, frees the buffer at +4 through FUN_00401c00 (alignment 4), zeroes +4, and frees this with _free when bit 0 of the flag byte is set. Returns this. |
