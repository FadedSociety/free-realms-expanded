# Class: GFormattedStringAccess

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00482308` | `GFormattedStringAccess::WriteGlobalAndOptionalArgument` | ✅ | virtual | Writes DAT_01af9bcc into *param_2 and, when param_3 is non-null, writes param_1 into *param_3. |
| `0048f5ea` | `GFormattedStringAccess::GFormattedStringAccess` | ✅ | ctor | Tiny constructor that installs the string-access vtables: +4 gets GStringAccess::vftable first, then both +0 and +4 are set to GFormattedStringAccess::vftable, i.e. the derived class overrides the base slot. |
| `0048f601` | `GFormattedStringAccess::InvokeSlotEC` | ✅ | virtual | Forwards to the receiver's virtual slot +0xEC with param_2 and the constant 1. |
| `0048f612` | `GFormattedStringAccess::Wrapper6` | ✅ | virtual | Normal 24-byte call wrapper: calls FUN_004c97db with literal 0, three incoming arguments and trailing 0, then returns; x86 epilogue uses ret 0x0c. This is not a variadic thunk. |
| `0048f62a` | `GFormattedStringAccess::Wrapper7` | ✅ | virtual | Normal 24-byte call wrapper: calls FUN_004c980a with literal 0, three incoming arguments and trailing 0, then returns; x86 epilogue uses ret 0x0c. This is not a variadic thunk. |
| `0048f642` | `GFormattedStringAccess::Wrapper8` | ✅ | virtual | Normal 40-byte call wrapper: calls FUN_004c9839 with literal 0, four incoming arguments and trailing 0, then returns; x86 epilogue uses ret 0x10. This is not a variadic thunk. |
| `0048f67f` | `GFormattedStringAccess::AdjustorThunkMinus4To0048f642` | ✅ | virtual | The 8-byte body subtracts 4 from ECX and jumps to 0048f642; it is an adjustor thunk, not a body call with unchanged arguments. |
| `0048f687` | `GFormattedStringAccess::AdjustorThunkMinus4To0048f612` | ✅ | virtual | The 8-byte body subtracts 4 from ECX and jumps to 0048f612. |
| `0048f68f` | `GFormattedStringAccess::AdjustorThunkMinus4To0048f717` | ✅ | virtual | The 8-byte body subtracts 4 from ECX and jumps to 0048f717. |
| `0048f697` | `GFormattedStringAccess::AdjustorThunkMinus4To00482308` | ✅ | virtual | The 8-byte body subtracts 4 from ECX and jumps to 00482308. |
| `0048f69f` | `GFormattedStringAccess::AdjustorThunkMinus4To0048f62a` | ✅ | virtual | The 8-byte body subtracts 4 from ECX and jumps to 0048f62a. |
| `0048f6a7` | `GFormattedStringAccess::AttachRangeChecked` | ✅ | virtual | Validates the named range through slot +0x6c using param_2, start param_4 and length param_5. On success it attaches {start=param_4,end=param_4+param_5} for nonnegative length or {start=param_4,end=0x7ffffffe} for negat… |
| `0048f717` | `GFormattedStringAccess::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: writes GInterface::vftable to the object's two vtable positions and frees param_1 only when param_2 bit 0 is set; the non-free path returns param_1. |
