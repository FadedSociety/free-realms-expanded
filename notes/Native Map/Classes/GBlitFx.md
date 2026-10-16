# Class: GBlitFx

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00405b36` | `GBlitFx::GBlitFx` | ✅ | ctor | Constructor: runs the GDrawFx base constructor (00405a32), overwrites the vtable at +0 and +4 with GBlitFx::vftable, zeroes +0x28 and +0x2C, then calls FUN_004d9a26(param_2, 0); returns this. |
| `0040d702` | `GBlitFx::VectorDeletingDtorThunk ? ⚠rename` | 🟡 | dtor | Eight-byte scalar-deleting-destructor adjustor thunk: subtracts 4 from ECX and jumps to FUN_0040e7a8, the GBlitFx deleting destructor body. |
| `0040e7a8` | `GBlitFx::ScalarDeletingDtor` | 🟢 | dtor | Scalar deleting destructor: calls the shared GInterface-derived teardown FUN_00526562, then frees the object through thunk_FUN_004c6a61 when the flag low bit is set. |
| `0041873b` | `GBlitFx::GBlitFx` | ✅ | ctor | Constructor of the blit draw-effect object: runs the GDrawFx base constructor FUN_00405a32, stores the GBlitFx vtable pair at +0/+4, sets the kind field at +0x18 to 3 and initializes the remaining effect fields (+0x14=0… |
| `00480689` | `GBlitFx::WriteGlobalAndOptionalArgument` | ✅ | virtual | Writes DAT_01af861c into *param_2 and, when param_3 is non-null, writes param_1 into *param_3. |
| `00480714` | `RegisterGBlitFx` | ✅ | init | Registers the GFC class "GBlitFx" (base size 0x18) with the class registry through FUN_0053da53/FUN_004ccf27/FUN_00536678. |
| `0048074d` | `GBlitFx::CopyFieldsChecked` | ✅ | virtual | Calls the guarded base copy FxCopyBaseFields and, when it succeeds, copies fields +0x24,+0x28,+0x2c,+0x30,+0x34 from param_2 to this, returning 1. |
