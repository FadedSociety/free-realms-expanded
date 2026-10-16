# Class: SoeUtil::Array<unsigned___int64,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008c7680` | `SoeUtil_ArrayUInt64_Dtor` | ✅ | dtor | Bytes: stores the Array<unsigned__int64,0,1> vtable at this+0, zeroes the count at this+8, calls FUN_00401c00 on the buffer at this+4 with alignment 8 (push 8), and zeroes this+4. Returns void. |
| `008c76d0` | `— ⚠reject` | ⚪ | virtual | Same reserve logic as the other element-array slot 1 functions (grow 5/4 or 6/5, keep within 4/3, zero-request handling). Otherwise writes the capacity to *param_3 and returns FUN_00401bb0(newcap*8, 8), an 8-byte-elemen… |
| `008c7770` | `SoeUtil_ArrayUInt64::VSlot3_SwapFields` | ✅ | virtual | Same body as SoeUtil_ArrayActorCompositeEffectTrigger::VSlot3_SwapFields (008c7370): with flag param_3 nonzero tail-jumps to vslot 3 of param_2; otherwise swaps the buffer pointer (+4) by value and the count (+8) and ca… |
| `008c92f0` | `SoeUtil_ArrayUInt64_ScalarDeletingDtor` | ✅ | dtor | Stores the Array<unsigned__int64,0,1> vtable at this+0, zeroes the count at +8, frees the buffer at +4 through FUN_00401c00 with alignment 8, zeroes +4, and frees this with _free when bit 0 of the flag byte is set. Retu… |
