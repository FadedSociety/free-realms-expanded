# Class: SoeUtil::List<Fotomat::NpcEntry,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bd500` | `FotomatNpcEntryList::AllocateNode` | ✅ | virtual | Initializes allocator state, tries thread and global allocation, then allocates 0x70 bytes aligned to 0x10. |
| `008bd540` | `FotomatNpcEntryList::FreeNode` | ✅ | virtual | Calls FreeMemory with the supplied pointer and 0x10-byte alignment, then returns. |
| `00910710` | `DtorBodyClearHead` | ✅ | dtor | Stores List vftable 0x01802034 at [esi]; then while [this+4] is nonzero calls FUN_008e47b0([this+4]) with ECX=this (callee pops its argument). Loop test precedes each call. |
| `009113b0` | `ScalarDeletingDtor → SoeUtil::ListFotomatNpcEntryScalarDtor` | ✅ | dtor | Virtual at vftable[0] (thiscall, one byte argument). Stores List<Fotomat::NpcEntry,-1> vftable at [this], runs the clear loop (while [this+4] != 0 call thiscall FUN_008e47b0([this+4])), then returns this when (arg & 1) … |
