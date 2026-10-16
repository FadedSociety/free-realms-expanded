# Class: GaqItemDuelRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a09710` | `GaqItemDuelRequest_ForwardToSlot0` | ✅ | virtual | Six-byte tail thunk that jumps to slot 0 of its own vtable (bytes 8b 01 8b 10 ff e2), i.e. to the duel responder that calls the encounter reply builder 00a97c10 with status 0. |
| `00a09730` | `GaqItemDuelRequest_Respond0` | ✅ | virtual | Twenty-seven-byte virtual method calling FUN_00a97c10 with status 0 and the item's two parameters; that callee builds a BaseEncounterPacket reply. |
| `00a09750` | `GaqItemDuelRequest_Respond1` | ✅ | virtual | Twenty-seven-byte virtual method calling FUN_00a97c10 with status 1 and the item's two parameters. |
