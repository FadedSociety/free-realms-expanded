# Class: GaqItemGroupInvite

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a090d0` | `GaqItemGroupInvite_Respond0` | ✅ | virtual | Eleven-byte virtual method that loads the action object at this+4 and calls FUN_00a983b0 with status 0. |
| `00a090e0` | `GaqItemGroupInvite_Respond1` | ✅ | virtual | Eleven-byte virtual method that loads the action object at this+4 and calls FUN_00a983b0 with status 1. |
| `00a090f0` | `GaqItemGroupInvite_ForwardToSlot0` | ✅ | virtual | Six-byte tail thunk: loads the object vtable and jumps to slot 0 of the same object (bytes 8b 01 8b 10 ff e2), so the action is the item's first responder, which calls the group-invite response builder 00a983b0. |
