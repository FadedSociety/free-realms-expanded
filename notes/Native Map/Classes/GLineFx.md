# Class: GLineFx

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0040e68c` | `GLineFx::GLineFx` | ✅ | ctor | Constructs a GLineFx effect: runs the base effect constructor FUN_00405a32 (GDrawFx), writes the two GLineFx vtable pointers at +0 and +4, then attaches the parameter object through FUN_004d9a26(param_2, 0). |
| `0040e6eb` | `GLineFx::VectorDeletingDtorThunk ? ⚠rename` | 🟡 | dtor | Eight-byte scalar-deleting-destructor adjustor thunk: subtracts 4 from ECX and jumps to FUN_0040e6f3, the GLineFx deleting destructor body. |
| `0040e6f3` | `GLineFx::ScalarDeletingDtor` | 🟢 | dtor | Scalar deleting destructor: calls the shared GInterface-derived teardown FUN_00526562, then, when the flag argument low bit is set, frees the object through the engine heap free thunk_FUN_004c6a61 and returns the pointe… |
| `004186c9` | `GLineFx::GLineFx` | ✅ | ctor | Constructor of the line draw-effect object: runs the GDrawFx base constructor FUN_00405a32, stores the GLineFx vtable pair at +0/+4, sets the kind field at +0x18 to 1 and initializes the remaining effect fields (+0x14=0… |
| `00480302` | `GLineFx::WriteGlobalAndOptionalArgument` | ✅ | virtual | Writes DAT_01af82a4 into *param_2 and, when param_3 is non-null, writes param_1 into *param_3. |
| `0048038d` | `RegisterGLineFx` | ✅ | init | Registers the GFC class "GLineFx" (base size 0x18) with the class registry through FUN_0053da53/FUN_004ccf27/FUN_00536678. |
| `004803c6` | `GLineFx::CopyFieldsCheckedPlus` | ✅ | virtual | Calls the guarded base copy FxCopyBaseFields and, when it succeeds, copies fields +0x24,+0x28,+0x2c,+0x30 from param_2 to this, returning 1. |
