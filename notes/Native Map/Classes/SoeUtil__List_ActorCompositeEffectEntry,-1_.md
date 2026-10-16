# Class: SoeUtil::List<ActorCompositeEffectEntry,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00910610` | `DtorBodyClearHead → SoeUtil::List<ActorCompositeEffectEntry,-1>::DtorBody` | ✅ | dtor | Stores List vftable 0x01804e54 at [esi]; then while [this+4] is nonzero calls thiscall FUN_008f62f0([this+4]) with ECX=this (callee pops its argument). Loop test precedes each call. |
| `00910650` | `AllocBlockSize52Align4` | 🟢 | virtual | Calls FUN_006fb9c0, then FUN_006fb860 with ECX = its result; pushes 4 then 0x34. If the returned object is non-null, calls its vtbl[1] with ECX=object and args (0x34,4) (callee pops) and returns that EAX. Else if DAT_01… |
| `00910690` | `FreeBlockAlign4` | 🟢 | virtual | Reads its one stack argument p, pushes 4 then p, calls FUN_00401c00(p,4), pops 8 bytes and returns with ret 4 (callee pops one argument). |
| `00911370` | `ScalarDeletingDtor → SoeUtil::ListActorCompositeEffectEntryScalarDtor` | ✅ | dtor | Virtual at vftable[0] (thiscall, one byte argument). Stores List<ActorCompositeEffectEntry,-1> vftable at [this], runs the same clear loop as the destructor body (while [this+4] != 0 call thiscall FUN_008f62f0([this+4])… |
