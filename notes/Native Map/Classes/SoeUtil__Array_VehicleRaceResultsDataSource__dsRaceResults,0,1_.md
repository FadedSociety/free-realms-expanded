# Class: SoeUtil::Array<VehicleRaceResultsDataSource::dsRaceResults,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ce29e0` | `ArrayDsRaceResultsGrowAlloc` | ✅ | virtual | Virtual slot [1] (returns the data pointer in eax): computes a new element count n from param_2 and capacity at +0xc (when the byte flag is zero it grows by 5/4 when n exceeds capacity and shrinks by 6/5 when capacity e… |
| `00ce2a80` | `ArrayDsRaceResultsFreeBlock` | ✅ | virtual | Virtual slot [2] (one stack argument, ret 8): pushes alignment 8 and the pointer argument and calls FUN_00401c00 (FreeMemory), then cleans its own pushes. |
| `00ce2aa0` | `ArrayDsRaceResultsSwapFields ?` | 🟡 | virtual | Virtual slot [3] (one stack pointer and one byte flag): when the flag is nonzero it calls vtbl+0xc of the pointed-to object; otherwise it exchanges the dwords at this+4, +8 and +0xc with those of the pointed-to object (… |
| `00ce2dd0` | `ArrayDsRaceResultsDtorBody` | ✅ | dtor | Thiscall destructor body (ret 0): stores the Array<dsRaceResults> vtable at this, calls FUN_00ce2c00 with the count at +8 (truncating all elements to zero), calls FUN_00401c00 to free the data block at +4 with alignment… |
| `00ce2e00` | `ArrayDsRaceResultsScalarDeletingDtor` | ✅ | dtor | Thiscall scalar deleting destructor (ret 4): stores the Array<dsRaceResults> vtable, truncates the elements through FUN_00ce2c00, frees the data block with FUN_00401c00 and nulls +4, then calls _free(this) when bit 0 of… |
