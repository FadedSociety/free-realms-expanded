# Class: SoeUtil::Array<ActorCompositeEffectTrigger,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bc630` | `FreeStorage` | ✅ | virtual | Calls FUN_00401c00 with the supplied pointer and alignment 4. |
| `008c7280` | `SoeUtil_ArrayActorCompositeEffectTrigger_Dtor` | ✅ | dtor | Stores SoeUtil::Array<ActorCompositeEffectTrigger,0,1>::vftable at this+0, zeroes the count at this+8, frees the buffer at this+4 through FUN_00401c00 with alignment 4, and zeroes this+4. Returns void. |
| `008c72d0` | `— ⚠reject` | ⚪ | virtual | Computes a new capacity from the current count at this+0xc and param_2: when param_4 is zero it grows by 5/4 or 6/5 or keeps the buffer at this+4 when the count is within 4/3 of the request; when param_4 is nonzero it r… |
| `008c7370` | `SoeUtil_ArrayActorCompositeEffectTrigger::VSlot3_SwapFields` | ✅ | virtual | Bytes: if the byte flag param_3 is nonzero, tail-jumps to vslot 3 (+0xc) of param_2 with this and 0 as arguments. Otherwise exchanges the buffer pointer (+4) between this and param_2 by value, and the count (+8) and cap… |
| `008c91b0` | `SoeUtil_ArrayActorCompositeEffectTrigger_ScalarDeletingDtor` | ✅ | dtor | Stores the Array<ActorCompositeEffectTrigger,0,1> vtable at this+0, zeroes the count at +8, frees the buffer at +4 through FUN_00401c00 (alignment 4), zeroes +4, and frees this with _free when bit 0 of the flag byte is … |
