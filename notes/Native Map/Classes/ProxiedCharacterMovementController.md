# Class: ProxiedCharacterMovementController

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bfba00` | `ProxiedCharacterMovementController::~ProxiedCharacterMovementController` | ✅ | dtor | Rewrites the object's first dword to ProxiedCharacterMovementController::vftable and returns; it releases no storage. |
| `00bfba80` | `ProxiedCharacterMovementController::ScalarDeletingDestructor` | ✅ | dtor | Restores the movement-controller vtable, tests param_2 bit 0, conditionally frees this, and returns the original pointer when it does not free. |
