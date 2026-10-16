# Class: GDrawFx

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00405a32` | `GDrawFx::GDrawFx` | ✅ | ctor | Constructor: runs the GObjectBase base constructor FUN_004d99d1, installs the GDrawFx vtable pointer into both +0 and +4, zeroes +0x1C and +0x20, and returns this. |
| `0040d6e2` | `GDrawFx::VectorDeletingDtorThunk ? ⚠rename` | 🟡 | dtor | Eight-byte scalar-deleting-destructor adjustor thunk: subtracts 4 from ECX (adjusting the secondary-interface this pointer back to the object base) and jumps to FUN_00405a94, the deleting destructor body. |
| `004801e5` | `GDrawFx::WriteGlobalAndOptionalArgument` | ✅ | virtual | Writes DAT_01af81c4 into *param_2 and, when param_3 is non-null, writes param_1 into *param_3. |
| `00480270` | `RegisterGDrawFx` | ✅ | init | Registers the GFC class "GDrawFx" (base size 0x18) with the class registry through FUN_0053da53/FUN_004ccf27/FUN_00536678. |
| `004802a3` | `— ⚠reject` | ⚪ | virtual | When the compatibility check FUN_004c6e7c accepts param_2, copies the four fields +0x14,+0x18,+0x1c,+0x20 from param_2 to this and returns 1; otherwise returns the check result. |
| `0048e0b4` | `GDrawFx::CopyCtor` | ✅ | ctor | Constructs a GDrawFx in param_1 from the source effect object param_2: runs FUN_004e68fb(param_2), installs the GDrawFx vtable into both +0 and +4, and copies the source fields +0x14, +0x18, +0x1C and +0x20 into +5, +6,… |
