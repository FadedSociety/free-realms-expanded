# Class: SoeUtil::List<InteractionData,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a37170` | `ListInteractionDataRouteAlloc40` | ✅ | virtual | Virtual slot [2] of SoeUtil::List<InteractionData,-1>::vftable (binary vtable read). Calls 006fb9c0, then 006fb860 for an allocator object: when non-null it calls that object's vtable slot +4 and returns. Otherwise, if … |
| `00a371b0` | `ListInteractionDataFreeAlign4` | ✅ | virtual | Virtual slot [3] of SoeUtil::List<InteractionData,-1>::vftable (binary vtable read). Body is a single call 00401c00(param_1, 4): the pointer argument with alignment 4 is passed to the memory free routine. |
| `00a37530` | `ListInteractionDataDtorBody` | ✅ | dtor | Bytes: mov dword [ecx], vftable(SoeUtil::List<InteractionData,-1>) then a jump (e9) to 00a37480. It jumps to 00a37480 (list clear, free every node through slot [3]) and does not return to this body itself. |
| `00a37580` | `ListInteractionDataScalarDtor` | ✅ | dtor | Virtual slot [0] of SoeUtil::List<InteractionData,-1>::vftable (binary vtable read). Body shape of a scalar deleting destructor: stores the list vtable into *this, calls 00a37480 (ListClearFreeAllNodes), then tests bit … |
