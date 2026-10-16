# Class: SoeUtil::List<ProxiedCharacter::Destination,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0095b130` | `List::AllocateNode` | ✅ | virtual | Initializes allocator state, obtains the current allocator, and calls its virtual slot +4 if present; otherwise it tries the global allocator at DAT_01bdf280 through slot +4. Only when neither allocator is available doe… |
| `0095b170` | `ListFreeNodeStorage` | ✅ | virtual | Calls FUN_00401c00 on the supplied pointer with alignment 0x10. |
| `00962850` | `ListProxiedDestinationDestructor` | ✅ | dtor | Installs the typed list vtable and calls FUN_0095e530 to clear the list. |
| `009628b0` | `DestinationListScalarDeletingDestructor` | ✅ | dtor | Restores the destination-list vtable, clears the list, and frees param_1 only when flag bit 0 of param_2 is set. |
