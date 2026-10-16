# Class: SoeUtil::List<Invitation,8>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0102ecb0` | `List::Ctor → SoeUtil::List<Invitation,8>::Ctor` | ✅ | ctor | Fastcall constructor (ECX = object): stores the List<Invitation,-1> vftable, zeroes +0x4, +0x8 and +0xc, stores the List<Invitation,8> vftable and calls 0102d560 (pool initialiser for eight 304-byte blocks). |
| `0102f6e0` | `Thunk_0102f600` | ✅ | virtual | Adjustor thunk (bytes: add ecx,0x10; jmp 0102f600): adds 0x10 to this and tail-jumps to 0102f600 (pool allocator). |
| `0102f6f0` | `Thunk_0102f670` | ✅ | virtual | Adjustor thunk (bytes: add ecx,0x10; jmp 0102f670): adds 0x10 to this and tail-jumps to 0102f670 (pool free routine). |
| `0102f700` | `List::DtorBody → SoeUtil::List<Invitation,8>::DtorBody` | ✅ | dtor | Fastcall destructor body (second copy of 0102ebe0 for the Invitation pool): stores the List<Invitation,8> vftable, drains the list via 0102de40 (RemoveNode) on the first node, frees the chunk tree via 0102d890 (children… |
| `0102f7f0` | `List::DeletingDtor → SoeUtil::List<Invitation,8>::DeletingDtor` | ✅ | dtor | Scalar deleting destructor (thiscall, flag byte): calls the pooled Invitation list destructor body 0102f700 on this, then calls _free(this) when bit 0 of the flag is set; returns this. |
