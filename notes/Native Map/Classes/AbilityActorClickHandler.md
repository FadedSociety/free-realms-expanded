# Class: AbilityActorClickHandler

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a2d950` | `AbilityActorClickHandler::MatchKind ⚠downgrade` | 🟢 | virtual | Returns 1 when the first argument is 8 and the second is 0, or when the first argument is 0x400, 0x800 or 0x1000; otherwise returns 0. |
| `00a2ef20` | `AbilityActorClickHandler::AbilityActorClickHandler` | ✅ | ctor | Runs the base click-handler constructor FUN_008c49a0, stores AbilityActorClickHandler::vftable at +0 and the argument at +0xc. |
| `00a2efd0` | `AbilityActorClickHandler::ScalarDeletingDtor` | 🟢 | dtor | Stores TargetProcessorEventHandler::vftable (the only vtable step of this destructor chain) and, when bit 0 of the flag argument is set, releases the object with _free; raw slot 0 of AbilityActorClickHandler::vftable at… |
| `00a359a0` | `AbilityActorClickHandler::DispatchEventToProcessor` | ✅ | virtual | Virtual slot [3] of AbilityActorClickHandler::vftable (binary vtable read). Event code is the stack argument at +0x18 (in_stack_00000018); param_3 is a pointer to four dwords. Code 0x8: takes the processor at this+0xc; … |
