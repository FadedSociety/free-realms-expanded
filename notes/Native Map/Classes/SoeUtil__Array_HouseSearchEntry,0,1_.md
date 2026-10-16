# Class: SoeUtil::Array<HouseSearchEntry,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa9550` | `SoeUtil::Array::Reserve` | ✅ | virtual | Virtual buffer-acquisition slot of the SoeUtil::Array instantiation: with param_4 false a request above the capacity at this+0xc is raised by 5/4 and a request below 3/4 of capacity is shrunk toward 6/5; with param_4 tr… |
| `00aa95f0` | `SoeUtil::Array::FreeStorage` | ✅ | virtual | Virtual release hook of the SoeUtil::Array instantiation: forwards (pointer, 8) to the shared free entry FUN_00401c00 (FreeMemory). |
| `00aa9610` | `SoeUtil::Array::SwapTripleAndNotify` | ✅ | virtual | Virtual slot [3] of the SoeUtil::Array instantiation: exchanges the three dwords at this+4/+8/+0xc with param_2[1..3] using XOR swaps; when param_3 is non-zero it instead tail-dispatches to param_2's vtable slot +0xc wi… |
| `00ab7d90` | `SoeUtil_Array_HouseSearchEntry_0_1::Dtor` | ✅ | dtor | Non-deleting destructor body: stores vftable 01818b04 at +0, calls FUN_00ab28e0 on +8, frees the +4 buffer through FUN_00401c00 with alignment 8, then nulls +4. |
| `00ab86b0` | `SoeUtil_Array_HouseSearchEntry_0_1::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0): stores vftable 01818b04, releases +8 through FUN_00ab28e0, frees the +4 buffer with FUN_00401c00 alignment 8, nulls +4, and calls _free on this when bit 0 of the flag argument… |
