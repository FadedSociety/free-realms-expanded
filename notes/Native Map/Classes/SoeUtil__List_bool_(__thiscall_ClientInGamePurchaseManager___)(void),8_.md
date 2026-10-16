# Class: SoeUtil::List<bool_(__thiscall_ClientInGamePurchaseManager::*)(void),8>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b30640` | `ClientInGamePurchaseManagerCallbackList::Ctor` | ✅ | ctor | Installs the list vtable, zeros its head, tail, and count, switches to the inline-capacity specialization, and initializes its storage subobject. |
| `00b306c0` | `— ⚠reject` | ⚪ | dtor | Drains list nodes, frees both endpoints of its backing allocation when present, clears fields, and restores the generic list vtable. |
| `00b30770` | `AdjustThisAndJump_00b30770 ? → ClientInGamePurchaseManagerCallbackList::Adjusto…` | 🟡 | virtual | Adds 0x10 to ECX, then jumps to 0x0070de20; it is an adjustor thunk and contains no separate method body. |
| `00b30780` | `AdjustThisAndJump_00b30780 ? → ClientInGamePurchaseManagerCallbackList::Adjusto…` | 🟡 | virtual | Adds 0x10 to ECX, then jumps to 0x0070de90; it is an adjustor thunk and contains no separate method body. |
| `00b30910` | `ClientInGamePurchaseManagerCallbackList::ScalarDeletingDtor` | ✅ | dtor | Calls the callback-list destructor, frees the object when flag bit 0 is set, and otherwise returns its pointer. |
