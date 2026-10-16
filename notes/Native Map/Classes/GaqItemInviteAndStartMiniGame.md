# Class: GaqItemInviteAndStartMiniGame

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a097b0` | `GaqItemInviteAndStartMiniGame_ForwardToSlot0` | ✅ | virtual | Six-byte tail thunk that jumps to slot 0 of its own vtable (bytes 8b 01 8b 10 ff e2), i.e. to the responder that passes the item parameters to the mini-game invite routine 00a97cc0 with status 0. |
| `00a097d0` | `GaqItemInviteAndStartMiniGame_Respond0 ⚠downgrade` | 🟢 | virtual | Virtual method passing the five item parameter dwords at +8..+0x18 plus status 0 to FUN_00a97cc0. |
| `00a09800` | `GaqItemInviteAndStartMiniGame_Respond1 ⚠downgrade` | 🟢 | virtual | Virtual method identical to the slot-0 responder but passing status 1 as the trailing argument to FUN_00a97cc0. |
