# Class: Engine::NetworkRequestSessionIDLogin

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0124f1e0` | `Engine::NetworkRequestSessionIDLogin::ctor` | 🟢 | ctor | Returns this. Stores Scaleform::IScaleformActionHandler::vftable then Engine::NetworkRequestSessionIDLogin::vftable into the object; when the int argument is nonzero it calls FUN_012ea4f0(this). |
| `0124f250` | `Engine::NetworkRequestSessionIDLogin::TypeName ⚠downgrade` | 🟢 | virtual | Slot [1] of Engine::NetworkRequestSessionIDLogin::vftable. Returns the constant C string 'RequestSessionIDLogin'. |
| `0124f270` | `Engine::NetworkRequestSessionIDLogin::DebugName` | ✅ | virtual | Slot [4] of its vftable. Returns the constant C string 'RequestSessionIDLogin : 0 : ..\common\coreclient\engine\network\networkhandlers.cc:130'. |
| `0124f280` | `Engine::NetworkRequestSessionIDLogin::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `0124f520` | `— ⚠reject` | ⚪ | virtual | Slot [2] of Engine::NetworkRequestSessionIDLogin::vftable. Calls FUN_012e0ba0, gets the thread-owned object from FUN_0124f2e0; if non-null it takes the C string from FUN_012eb250('SessionID', DAT_0175b400), copies it to… |
