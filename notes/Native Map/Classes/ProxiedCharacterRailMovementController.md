# Class: ProxiedCharacterRailMovementController

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bfbaa0` | `ProxiedCharacterRailMovementController::AdvanceRail` | ✅ | virtual | Adds the tick delta to progress at this+0x20 and clamps it to the rail length. Computes the rail transform, calls the controlled object's virtual+0x20, updates orientation, and frees/nulls the route object when progress… |
| `00bfbca0` | `— ⚠reject` | ⚪ | ctor | Installs the base and rail-controller vtables, stores its constructor arguments and transform, clears progress, then initializes route state and runs one update when the route lookup is valid and nonempty. |
| `00bfbd70` | `ProxiedCharacterRailMovementController::~ProxiedCharacterRailMovementController` | ✅ | dtor | Restores the rail vtable, calls the cleanup helper and frees the owned pointer at this+0x24 when non-null, then restores the base movement-controller vtable. |
| `00bfbde0` | `ProxiedCharacterRailMovementController::ScalarDeletingDestructor` | ✅ | dtor | Calls the rail destructor body, frees this only when param_2 bit 0 is set, and otherwise returns the original pointer. |
