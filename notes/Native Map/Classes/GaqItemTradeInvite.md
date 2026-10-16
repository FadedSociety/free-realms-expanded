# Class: GaqItemTradeInvite

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a09120` | `GaqItemTradeInvite_Respond0 ⚠downgrade` | 🟢 | virtual | Eleven-byte virtual method that forwards to FUN_00c6dbe0 with status 0. |
| `00a09130` | `GaqItemTradeInvite_Respond1 ⚠downgrade` | 🟢 | virtual | Eleven-byte virtual method that forwards to FUN_00c6dbe0 with status 1. |
| `00a09140` | `GaqItemTradeInvite_ForwardToSlot0` | ✅ | virtual | Six-byte tail thunk that jumps to slot 0 of its own vtable (bytes 8b 01 8b 10 ff e2), i.e. to the trade-invite responder that calls 00c6dbe0 with status 0. |
