# Class: Engine::NetworkRequestPasswordLogin

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0124e7e0` | `Engine::NetworkRequestPasswordLogin::ctor` | 🟢 | ctor | Returns this. Stores Scaleform::IScaleformActionHandler::vftable and then Engine::NetworkRequestPasswordLogin::vftable into the object; when the int argument is nonzero it calls FUN_012ea4f0(this). |
| `0124e850` | `Engine::NetworkRequestPasswordLogin::TypeName ⚠downgrade` | 🟢 | virtual | Slot [1] of Engine::NetworkRequestPasswordLogin::vftable. Returns the constant C string 'RequestPasswordLogin'. |
| `0124e870` | `Engine::NetworkRequestPasswordLogin::DebugName` | ✅ | virtual | Slot [4] of its vftable. Returns the constant C string 'RequestPasswordLogin : 0 : ..\common\coreclient\engine\network\networkhandlers.cc:144'. |
| `0124e880` | `Engine::NetworkRequestPasswordLogin::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `0124f610` | `Engine::NetworkRequestPasswordLogin::ExecuteCredentialFields` | 🟢 | virtual | Slot [2] of Engine::NetworkRequestPasswordLogin::vftable. Calls FUN_012e0ba0, gets the thread-owned object from FUN_0124f2e0; if non-null it takes FUN_012eb250('UserName', DAT_0175b400) and FUN_012eb250('Password', DAT_… |
