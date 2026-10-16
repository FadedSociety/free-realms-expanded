# Class: ProxiedCharacterSeekMovementController

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bfc190` | `ProxiedCharacterSeekMovementController::ResetTargetData` | ✅ | virtual | Forms a two-word argument pair from the caller's stack values, adds the constant pointer 0x017efda8, and delegates through Target at this+0x1c to 0101c060. |
| `00bfc280` | `ProxiedCharacterSeekMovementController::ProxiedCharacterSeekMovementController` | ✅ | ctor | Installs the base and seek-controller vtables, initializes the embedded Target at this+0x1c through 0101c060, stores its arguments, and derives initial orientation from the supplied four-component vector. |
| `00bfc3e0` | `ProxiedCharacterSeekMovementController::~ProxiedCharacterSeekMovementController` | 🟢 | dtor | Calls Target cleanup 0101b370 with ECX=this+0x1c, then restores the base ProxiedCharacterMovementController vtable. |
| `00bfc440` | `ProxiedCharacterSeekMovementController::ScalarDeletingDestructor` | ✅ | dtor | Calls the seek-controller cleanup body, frees this only when param_2 bit 0 is set, and otherwise returns the original pointer. |
| `00bfc460` | `ProxiedCharacterSeekMovementController::Update` | ✅ | virtual | Clamps the controller's speed at this+0x24 using acceleration at this+0x0c and limit at this+0x10. With a valid controlled object, computes normalized target-relative vectors and applies a new transform through its virt… |
