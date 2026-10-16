# Class: SoeUtil::HashList<SoeUtil::IString<char>,16,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa9230` | `SoeUtil::HashList::Allocate` | ✅ | virtual | Virtual allocation hook shared by the SoeUtil::HashListMap and SoeUtil::HashList instantiations for IString keys: routes a fixed (0x20, 4) request to the active memory manager (FUN_006fb9c0 then FUN_006fb860, virtual sl… |
| `00aa9270` | `SoeUtil::HashList::Free` | ✅ | virtual | Virtual release hook shared by the HashListMap and HashList instantiations for IString keys: forwards (pointer, 4) to the shared free entry FUN_00401c00 (FreeMemory). |
| `00abc380` | `SoeUtil_HashList_IString_16_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0 of the HashList<IString<char>,16,-1> base): calls FUN_00abaad0 on this, then calls _free when bit 0 of the flag argument is set, and returns this. |
