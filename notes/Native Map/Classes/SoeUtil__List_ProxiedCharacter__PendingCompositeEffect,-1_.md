# Class: SoeUtil::List<ProxiedCharacter::PendingCompositeEffect,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0095b640` | `List::AllocateNode` | ✅ | virtual | Initializes allocator state, obtains the current allocator, and calls its virtual slot +4 if present; otherwise it tries the global allocator at DAT_01bdf280 through slot +4. Only when neither allocator is available doe… |
| `0095b680` | `ListFreeNode` | ✅ | virtual | Calls FUN_00401c00 with the supplied pointer and alignment 0x10. |
| `0096c460` | `ListPendingCompositeEffectDtorBody` | 🟢 | dtor | Fastcall destructor body: stores SoeUtil::List<ProxiedCharacter::PendingCompositeEffect,-1>::vftable at this, then drains the head at this+4 by calling 009684b0 with the head until it is zero. |
