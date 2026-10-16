# Class: FreeRealmsT4

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00979d60` | `FreeRealmsT4::FreeRealmsT4 ?` | 🟡 | ctor | Calls FUN_00e523f0(param_2, param_3, param_4) (a base initialiser taking three arguments) and then stores FreeRealmsT4::vftable at this+0. Returns this. |
| `00979dd0` | `— ⚠reject` | ⚪ | dtor | Stores FreeRealmsT4::vftable at this+0, stores this in a stack local, then calls FUN_00e52190 with the stack-cookie-derived value (cookie check shape). Returns nothing. |
| `00979e20` | `— ⚠reject` | ⚪ | dtor | Stores FreeRealmsT4::vftable at this+0, calls FUN_00e52190 with the stack-cookie-derived value, then frees the object with _free when bit 0 of the flag argument is set; returns this. |
