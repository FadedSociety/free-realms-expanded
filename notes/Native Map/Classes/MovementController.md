# Class: MovementController

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b16c00` | `MovementController::SetField3A` | ✅ | virtual | One-instruction virtual setter shared by the MovementController, InterpolatedMovementController and ClientInterpolatedMovementController vtables at slot [6]: raw bytes 8a 44 24 04 88 41 3a c2 04 00 store the incoming by… |
| `00b16c10` | `MovementController::GetField39` | ✅ | virtual | One-instruction virtual getter shared by the three movement controller vtables at slot [7]: raw bytes 8a 41 39 c3 return the byte field at this+0x39. |
| `00b16c20` | `MovementController::SetFlag39` | ✅ | virtual | One-instruction virtual setter shared by the three movement controller vtables at slot [10]: raw bytes c6 41 39 01 c3 store the constant 1 into the byte field at this+0x39 and return. |
