# Class: ControllerFreeFly

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ae85f0` | `ControllerFreeFly::DestroyControllerBaseState` | ✅ | dtor | Installs the ControllerFreeFly vtable and invokes the Controller base destructor to release its inherited key and controller state. |
| `00ae8650` | `ControllerFreeFly::GetFieldE4` | ✅ | virtual | Returns the float stored at this+0xe4. |
| `00ae8660` | `ControllerFreeFly::SetFieldE4` | ✅ | virtual | Stores the supplied float at this+0xe4 and returns. |
| `00ae8680` | `ControllerFreeFly::GetFieldE0` | ✅ | virtual | Returns the float stored at this+0xe0. |
| `00ae8690` | `ControllerFreeFly::Ctor` | ✅ | ctor | Constructs the Controller base, installs the ControllerFreeFly vtable, clears state flags, initializes basis vectors from constants, and registers several input/action codes. |
| `00ae87b0` | `ControllerFreeFly::ScalarDeletingDtor` | ✅ | dtor | Calls the Controller base destructor, frees the object when flag bit 0 is set, and returns its pointer otherwise. |
| `00ae8810` | `ControllerFreeFly::GetHorizontalBasisAngle` | ✅ | virtual | Reads two horizontal basis components from the associated controller transform, returns atan2 of them when their squared magnitude exceeds the threshold, and otherwise returns zero. |
| `00ae89c0` | `ControllerFreeFly::ApplyTransformWithHeadingCorrection ? ⚠rename` | 🟡 | virtual | Reads the controller transform, applies the supplied transform to its basis, corrects heading when the horizontal components exceed a threshold, adds the current world position, and submits the resulting transform. |
| `00ae8b30` | `ControllerFreeFly::ApplyBasisTransform ? ⚠rename` | 🟡 | virtual | Reads basis and position fields from the controller transform, applies the supplied transform, conditionally corrects the horizontal basis, and passes the resulting transform to 00792e20. |
| `00ae8c60` | `ControllerFreeFly::UpdateTransform` | ✅ | virtual | Reads the current transform, applies input-derived rotations and movement, normalizes and adjusts the resulting basis, invokes mode-dependent controller actions, then writes the updated transform. |
| `00ae9460` | `ControllerFreeFly::MoveFromInput` | ✅ | virtual | Reads registered movement inputs, derives forward/side/up axes and speed, applies transform basis vectors, clamps the target position to per-axis bounds, optionally resolves convex collisions, and submits the resulting … |
