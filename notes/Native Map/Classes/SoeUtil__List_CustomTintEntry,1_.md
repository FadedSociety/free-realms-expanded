# Class: SoeUtil::List<CustomTintEntry,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00799cd0` | `ListCustomTintEntryInlineCtor` | ✅ | ctor | Fastcall (ecx = this). Stores SoeUtil::List<CustomTintEntry,-1>::vftable, zeroes +4, +8, +0xc, +0x10, +0x14 and the byte at +0x20, stores SoeUtil::List<CustomTintEntry,1>::vftable at +0, points +0x18 at the inline buffe… |
| `0079a710` | `ListCustomTintEntry1AllocThunk` | 🟢 | virtual | Adjustor thunk (bytes 83 c1 10; e9 rel32): adds 0x10 to ecx and jumps to 0079a630 (PoolEntryPop). Vtable slot [2] of SoeUtil::List<CustomTintEntry,1>. |
| `0079a720` | `ListCustomTintEntry1FreeThunk` | 🟢 | virtual | Adjustor thunk (bytes 83 c1 10; e9 rel32): adds 0x10 to ecx and jumps to 0079a6a0 (PoolEntryPush). Vtable slot [3] of SoeUtil::List<CustomTintEntry,1>. |
| `0079a730` | `ListCustomTintEntry1DtorBody` | ✅ | dtor | Fastcall (ecx = this). Stores SoeUtil::List<CustomTintEntry,1>::vftable at +0, unlinks and frees the head nodes through 00799670 until the head at +4 is NULL, frees the block tree at +0x10 through 00799340 with alignmen… |
| `0079a800` | `ListCustomTintEntry1ScalarDtor` | ✅ | dtor | Thiscall (ecx = this, one flag argument). Calls 0079a730 on this, frees this through _free when bit 0 of the flag is set, and returns this. |
