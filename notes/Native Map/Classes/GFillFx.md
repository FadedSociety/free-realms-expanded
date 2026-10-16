# Class: GFillFx

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00405ab0` | `GFillFx::GFillFx` | ✅ | ctor | Constructor: runs the GDrawFx base constructor (00405a32), overwrites the vtable at +0 and +4 with GFillFx::vftable, then calls FUN_004d9a26(param_2, 0); returns this. |
| `0040d742` | `GFillFx::VectorDeletingDtorThunk ? ⚠rename` | 🟡 | dtor | Eight-byte scalar-deleting-destructor adjustor thunk: subtracts 4 from ECX and jumps to FUN_0040e740, the GFillFx deleting destructor body. |
| `0040e740` | `GFillFx::ScalarDeletingDtor` | 🟢 | dtor | Scalar deleting destructor: calls the shared GInterface-derived teardown FUN_00526562, then frees the object through thunk_FUN_004c6a61 when the flag low bit is set. |
| `00418700` | `GFillFx::GFillFx` | ✅ | ctor | Constructor of the fill draw-effect object: runs the GDrawFx base constructor FUN_00405a32, stores the GFillFx vtable pair at +0/+4, sets the kind field at +0x18 to 2 and initializes the remaining effect fields (+0x14=0… |
| `00480536` | `GFillFx::WriteGlobalAndOptionalArgument` | ✅ | virtual | Writes DAT_01af84c4 into *param_2 and, when param_3 is non-null, writes param_1 into *param_3. |
| `004805c1` | `RegisterGFillFx` | ✅ | init | Registers the GFC class "GFillFx" (base size 0x18) with the class registry through FUN_0053da53/FUN_004ccf27/FUN_00536678. |
| `004805fa` | `GFillFx::CopyFieldsAndBlockChecked` | ✅ | virtual | Calls the guarded base copy FxCopyBaseFields, copies fields +0x24,+0x28,+0x2c,+0x30 from param_2 and memcpy's the 0x20 bytes at +0x34, returning the copy status. |
