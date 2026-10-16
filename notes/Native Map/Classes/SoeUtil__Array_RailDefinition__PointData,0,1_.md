# Class: SoeUtil::Array<RailDefinition::PointData,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bd810` | `RailPointDataArray::FreeBuffer` | ✅ | virtual | Releases the supplied array buffer with 4-byte alignment. |
| `008c7520` | `SoeUtil_ArrayRailPointData_Dtor` | ✅ | dtor | Stores SoeUtil::Array<RailDefinition::PointData,0,1>::vftable at this+0, zeroes the count at this+8, frees the buffer at this+4 through FUN_00401c00 with alignment 4, and zeroes this+4. Returns void. |
| `008c7570` | `SoeUtil_ArrayRailPointData::VSlot1_ReserveBuffer` | ✅ | virtual | Computes a new capacity from the count at this+0xc and param_2 exactly as the other reserve slots: when param_4 is zero it grows by 5/4 or 6/5 or keeps the buffer at this+4 when the count is within 4/3 of the request; w… |
| `008c7610` | `SoeUtil_ArrayRailPointData::VSlot3_SwapFields` | ✅ | virtual | Same body as SoeUtil_ArrayActorCompositeEffectTrigger::VSlot3_SwapFields (008c7370): with flag param_3 nonzero tail-jumps to vslot 3 of param_2; otherwise swaps the buffer pointer (+4) by value and the count (+8) and ca… |
| `008c92b0` | `SoeUtil_ArrayRailPointData_ScalarDeletingDtor` | ✅ | dtor | Stores the Array<RailDefinition::PointData,0,1> vtable at this+0, zeroes the count at +8, frees the buffer at +4 through FUN_00401c00 (alignment 4), zeroes +4, and frees this with _free when bit 0 of the flag byte is se… |
