# Class: SoeUtil::List<ActorGroupEntry,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00799120` | `ListAllocBufferActorGroupEntry` | 🟢 | virtual | Vtable slot [2] of SoeUtil::List<ActorGroupEntry,-1>, no stack arguments. Calls 006fb9c0, then gets an object pointer from 006fb860; when it is non-NULL calls that manager's slot [1] with (0x80, 0x10) pushed; otherwise … |
| `00799160` | `ListFreeBuffer` | 🟢 | virtual | Vtable slot [3] of SoeUtil::List<ActorGroupEntry,-1>, one stack argument (ret 4). Calls 00401c00(argument, 0x10) to free the buffer with 16-byte alignment. |
| `0079a990` | `ListActorGroupEntryDtorBody` | ✅ | dtor | Fastcall (ecx = this). Stores SoeUtil::List<ActorGroupEntry,-1>::vftable at +0, then repeatedly unlinks and frees the head node through 0079a8d0 until the head is NULL. |
