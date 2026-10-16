# Class: AbilityTerrainClickHandler

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a2d910` | `AbilityTerrainClickHandler::MatchKind` | ✅ | virtual | Returns 1 when the first argument is 8 and the second is 0, or when the first argument is 0x400, 0x800 or 0x1000; otherwise returns 0. |
| `00a2f6e0` | `AbilityTerrainClickHandler::AbilityTerrainClickHandler` | ✅ | ctor | Runs the base constructor FUN_008b3590 (which stores TargetProcessorEventHandler::vftable, zeroes +4/+8 and then stores TargetProcessorTileEventHandler::vftable), stores AbilityTerrainClickHandler::vftable at +0 and the… |
| `00a2f790` | `AbilityTerrainClickHandler::ScalarDeletingDtor` | 🟢 | dtor | Stores TargetProcessorEventHandler::vftable (the only vtable step of this destructor chain) and, when bit 0 of the flag argument is set, releases the object with _free; raw slot 0 of AbilityTerrainClickHandler::vftable … |
| `00a35920` | `AbilityTerrainClickHandler::DispatchEventToProcessor` | ✅ | virtual | Virtual slot [3] of AbilityTerrainClickHandler::vftable (binary vtable read). Takes an event code in its sixth argument (param_6) and a pointer to four dwords (param_2). Code 0x8: calls 00a35810 on the processor object … |
