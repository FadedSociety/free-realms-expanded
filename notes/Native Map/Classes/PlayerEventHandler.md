# Class: PlayerEventHandler

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a8a40` | `MatchPlayerEventKind_8_10_80_400_800` | ✅ | virtual | Predicate on one integer: returns 1 exactly when the argument is 8, 0x10, 0x80, 0x400 or 0x800, otherwise 0. No calls, no memory access. Installed as slot 2 of PlayerEventHandler::vftable. |
| `008b8830` | `PlayerEventHandler::PlayerEventHandler` | ✅ | ctor | Calls TargetProcessorPlayerEventHandler construction, installs PlayerEventHandler::vftable, and stores param_2 at +0x0c. |
| `0093f1c0` | `PlayerEventHandler::HandleEvent` | ✅ | handler | Branches on masks 0x8, 0x10, 0x80, 0x400, and 0x800; creates or updates Target records, compares target keys, and calls player-event and target callbacks. |
