# Class: ProxiedCharacterRelativeRailMovementController

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bfbe10` | `ProxiedCharacterRelativeRailMovementController::AdvanceRelativeRail` | ✅ | virtual | Advances the two progress values at this+0x38 and this+0x3c, clamped to their route lengths. Interpolates the paired route position, applies the transform through virtual+0x20, updates orientation, and frees/nulls both … |
| `00bfbfb0` | `ProxiedCharacterRelativeRailMovementController::ProxiedCharacterRelativeRailMov…` | ✅ | ctor | Installs the base and relative-rail vtables, stores its route references, additional constructor values, and a four-word vector, initializes progress to zero, and sets up route state when both lookups and flags permit. |
| `00bfc0e0` | `ProxiedCharacterRelativeRailMovementController::~ProxiedCharacterRelativeRailMo…` | ✅ | dtor | Restores the relative-rail vtable, cleans and frees the pointers at this+0x30 and this+0x34 when non-null, then restores the base movement-controller vtable. |
| `00bfc170` | `ProxiedCharacterRelativeRailMovementController::ScalarDeletingDestructor` | ✅ | dtor | Calls the relative-rail destructor body, frees this only when param_2 bit 0 is set, and otherwise returns the original pointer. |
