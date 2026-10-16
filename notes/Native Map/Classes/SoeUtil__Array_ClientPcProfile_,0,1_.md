# Class: SoeUtil::Array<ClientPcProfile*,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008be8e0` | `ClientPcProfilePointerArray::Allocate` | ✅ | virtual | Computes a resized capacity, reuses old storage when eligible, or allocates capacity times 4 bytes aligned to 4. |
| `008be980` | `ClientPcProfilePointerArray::FreeBuffer` | ✅ | virtual | Releases the supplied pointer-array buffer with 4-byte alignment. |
| `008be9a0` | `ClientPcProfilePointerArray::DispatchOrSwap` | ✅ | virtual | Dispatches the entry callback when flagged; otherwise swaps entry words at +4, +8 and +0xc. |
| `008c7f70` | `SoeUtil_ArrayClientPcProfilePtr_Dtor` | ✅ | dtor | Stores SoeUtil::Array<ClientPcProfile*,0,1>::vftable at this+0, zeroes the count at this+8, frees the buffer at this+4 through FUN_00401c00 with alignment 4, and zeroes this+4. Returns void. |
| `008c94b0` | `SoeUtil_ArrayClientPcProfilePtr_ScalarDeletingDtor` | ✅ | dtor | Stores the Array<ClientPcProfile*,0,1> vtable at this+0, frees the buffer at +4 through FUN_00401c00 (alignment 4), zeroes +4, and frees this with _free when bit 0 of the flag byte is set. Returns this. |
