# Class: SoeUtil::List<CharacterStatSetDefinition::StatEntry,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0102bf80` | `List::AllocNode16` | ✅ | virtual | Virtual with no stack args: calls 006fb9c0, then gets the current thread memory manager (006fb860); if present pushes (16,4) and calls its slot [1]; else uses the global manager (DAT_01bdf280) slot [1]; else calls 006fb… |
| `0102bfc0` | `List::FreeNode → SoeUtil::List<CharacterStatSetDefinition::StatEntry,-1>::FreeN…` | ✅ | virtual | Thiscall virtual with one stack argument (ret 4): calls FreeMemory 00401c00 on the argument with alignment 4 and returns. |
| `0102c550` | `List::Clear → SoeUtil::List<CharacterStatSetDefinition::StatEntry,-1>::Clear` | 🟢 | method | Thiscall (ECX = list): while list+0x4 (first) is non-null, unlinks that first node and calls the list vtable slot [3] on the node. |
| `0102c610` | `List::DtorBody → SoeUtil::List<CharacterStatSetDefinition::StatEntry,-1>::DtorB…` | ✅ | dtor | Stores the StatEntry list vftable at this (ECX) and tail-jumps to 0102c550 (list clear); no free. |
| `0102c620` | `List::DeletingDtor → SoeUtil::List<CharacterStatSetDefinition::StatEntry,-1>::D…` | ✅ | dtor | Scalar deleting destructor (thiscall, flag byte argument): stores the StatEntry list vftable, calls 0102c550 (clear), calls _free(this) when bit 0 of the flag is set; returns this. |
