# Class: ClientPromotionalManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a05b90` | `ClientPromotionalManager::LogLevel5 ⚠downgrade` | 🟢 | virtual | Virtual log method: forwards to the logging service FUN_00700f90 with severity 5, the fixed file name Promotions.log and the caller's varargs. |
| `00a05bb0` | `ClientPromotionalManager::LogLevel4` | ✅ | virtual | Virtual log method: forwards to FUN_00700f90 with severity 4, file name Promotions.log and the caller's varargs. |
| `00a05bd0` | `ClientPromotionalManager::LogLevel3` | ✅ | virtual | Virtual log method: forwards to FUN_00700f90 with severity 3, file name Promotions.log and the caller's varargs. |
| `00a05bf0` | `ClientPromotionalManager::LogLevel2 ⚠downgrade` | 🟢 | virtual | Virtual log method: forwards to FUN_00700f90 with severity 2, file name Promotions.log and the caller's varargs. |
| `00a07500` | `ClientPromotionalManager::~ClientPromotionalManager` | ✅ | dtor | Manager destructor: installs the manager vtable, walks the list of registered objects at +0x310 (value 0x1C4 of the manager) unregistering each id through FUN_0081ad80 and releasing its reference count, drains the datas… |
| `00a07b80` | `ClientPromotionalManager::ClientPromotionalManager` | ✅ | ctor | Manager constructor: installs the ClientPromotionalManager vtable, stores the owner pointer at +4, runs the base constructor 01062450, sets the byte flag at +0x1F4 to 1, initialises one member with 00c6a720, then constr… |
