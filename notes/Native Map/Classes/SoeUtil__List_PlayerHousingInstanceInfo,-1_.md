# Class: SoeUtil::List<PlayerHousingInstanceInfo,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa91d0` | `SoeUtil::List::Allocate` | ✅ | virtual | Virtual allocation hook of the SoeUtil::List instantiation: routes a fixed (0xa0, 8) request to the active memory manager (FUN_006fb9c0 then FUN_006fb860, virtual slot +4), else the global manager DAT_01bdf280, else FUN… |
| `00aa9210` | `SoeUtil::List::Free` | ✅ | virtual | Virtual release hook of the SoeUtil::List instantiation: forwards (pointer, 8) to the shared free entry FUN_00401c00 (FreeMemory). |
| `00abc4b0` | `SoeUtil_List_PlayerHousingInstanceInfo_Dtor` | ✅ | dtor | Destructor body: stores the List<PlayerHousingInstanceInfo,-1> vftable 01818ab8 at +0 and drains the list at +4 through FUN_00ab6dd0 while it is nonzero. |
| `00abc860` | `SoeUtil_List_PlayerHousingInstanceInfo_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0): stores vftable 01818ab8, drains the list at +4 through FUN_00ab6dd0 while nonzero, then calls _free on this when bit 0 of the flag argument is set. |
