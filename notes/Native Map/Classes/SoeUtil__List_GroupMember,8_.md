# Class: SoeUtil::List<GroupMember,8>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0102e190` | `— ⚠reject` | ⚪ | ctor | Fastcall constructor (ECX = object): stores the List<GroupMember,-1> vftable, zeroes +0x4, +0x8 and +0xc, stores the List<GroupMember,8> vftable, then calls 0102d460 (pool initialiser) on the object. |
| `0102ebc0` | `Thunk_0102eae0 → GroupMemberPoolList::AllocNodeThunk` | ✅ | virtual | Adjustor thunk (bytes: add ecx,0x10; jmp 0102eae0): adds 0x10 to this and tail-jumps to the pool allocator 0102eae0. |
| `0102ebd0` | `Thunk_0102eb50 → GroupMemberPoolList::FreeNodeThunk` | ✅ | virtual | Adjustor thunk (bytes: add ecx,0x10; jmp 0102eb50): adds 0x10 to this and tail-jumps to the pool free routine 0102eb50. |
| `0102ebe0` | `List::DtorBody → SoeUtil::List<GroupMember,8>::DtorBody` | ✅ | dtor | Fastcall destructor body (ECX = pooled list): stores the List<GroupMember,8> vftable, drains the list (008faac0 on each node), frees the chunk tree rooted at +0x10 through 0102d850 (children +0x8/+0xc) and FreeMemory 00… |
| `0102f7d0` | `List::DeletingDtor → SoeUtil::List<GroupMember,8>::DeletingDtor` | ✅ | dtor | Scalar deleting destructor (thiscall, flag byte): calls the pooled GroupMember list destructor body 0102ebe0 on this, then calls _free(this) when bit 0 of the flag is set; returns this. |
