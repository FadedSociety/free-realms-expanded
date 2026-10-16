# Class: Engine::NetworkSynchronizationCompleteInstancesDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0124e4f0` | `Engine::NetworkSynchronizationCompleteInstancesDisplayAction::ctor` | 🟢 | ctor | Returns this. Stores the base IDisplayActionHandler vftable then Engine::NetworkSynchronizationCompleteInstancesDisplayAction::vftable; when the int argument is nonzero it calls FUN_012ea300(this). |
| `0124e580` | `Engine::NetworkSynchronizationCompleteInstancesDisplayAction::DebugName` | ✅ | virtual | Slot [4] of its vftable. Returns the constant C string 'kSynchronizationCompleteInstancesDisplayAction : 1 : ..\common\coreclient\engine\network\networkhandlers.cc:106'. |
| `0124e590` | `Engine::NetworkSynchronizationCompleteInstancesDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this; when bit 0 of the flag is set, calls _free(this) via the 00d82463 thunk; returns this. |
| `0124f370` | `Engine::NetworkSynchronizationCompleteInstancesDisplayAction::SetCompleteFlag` | 🟢 | virtual | Slot [2] of its vftable. Calls FUN_012e0ba0, then FUN_0124f2e0 (thread-owned object lookup). When that returns a non-null object it calls FUN_012adee0, uses its result as this for FUN_012ae8c0, and if that returns false… |
