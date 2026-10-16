# Class: GFlashFx

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `004595b0` | `GFlashFx::GFlashFx` | ✅ | ctor | Runs the GDrawFx base constructor (00405a32), zeroes the field at +0x24, installs GFlashFx::vftable at +0 and +4 and stores the constant 6 at +0x18; returns this. |
| `004595f2` | `GFlashFx::Slot2Thunk5017F8` | ✅ | virtual | Three-argument forwarder to FUN_005017f8: raw bytes ff 74 24 08 / ff 74 24 08 / 51 / e8 f8 81 0a 00 / 83 c4 0c / c2 08 00 push the two stack arguments and ECX and call the helper; the GFlashFx vtable entry at 0x17fa53c … |
| `0045960b` | `GFlashFx::GFlashFx_2` | 🟢 | ctor | Runs GDrawFx constructor 00405a32 with ECX=this, installs the GFlashFx vtable at +0 and +4, pushes 0 then param_2 and calls 004d9a26 with ECX=this, then returns this. |
| `00480431` | `GFlashFx::WriteGlobalAndOptionalArgument` | ✅ | virtual | Writes DAT_01af8414 into *param_2 and, when param_3 is non-null, writes param_1 into *param_3. |
| `004804bc` | `RegisterGFlashFx` | ✅ | init | Registers the GFC class "GFlashFx" (base size 0x18) with the class registry through FUN_0053da53/FUN_004ccf27/FUN_00536678. |
| `004804ef` | `— ⚠reject` | ⚪ | virtual | Calls the guarded base copy FxCopyBaseFields and, when it succeeds, copies field +0x24 from param_2 to this, returning 1. |
| `0048e38d` | `— ⚠reject` | ⚪ | ctor | Class-factory dispatch for GFlashFx keyed by param_1: modes 0/1 construct in place into param_2 (base constructor FUN_00405a32, GFlashFx vtables, field +9 zeroed and +6 set to 6), modes 2/3 allocate 0x28 bytes with FUN_… |
