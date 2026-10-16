# Class: SoundOneShotDefinitionLoader

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00915510` | `— ⚠reject` | ⚪ | ctor | __fastcall-style thiscall (this=arg1, arg2 on stack; returns this). Constructs its bases and members: calls FUN_008a2530 (SoundDefinitionAssetLoader constructor, label accepted:confirmed), FUN_008f4e10, and FUN_0090f900… |
| `009155a0` | `ThunkThisMinus8ToDeletingDtor` | 🟢 | virtual | Adjustor thunk (bytes 83 e9 08 e9 98 00 00 00): subtracts 8 from ECX and jumps to FUN_00915640. It is vftable[0] of the secondary vtable 01805060 of SoundOneShotDefinitionLoader. |
| `009155b0` | `DtorBody` | 🟢 | dtor | __fastcall (this=ECX). Under an SEH frame: restores the SoundOneShotDefinitionLoader vtables (01805074, 01805060, 01805054) and calls three cleanup helpers FUN_007dfc60, FUN_008ebd30 and FUN_008fa260 (receivers not veri… |
| `00915640` | `ScalarDeletingDtor → SoundOneShotDefinitionLoaderScalarDeletingDtor` | ✅ | dtor | Virtual at vftable[0] of 01805074 (thiscall, one byte argument). Calls FUN_009155b0 (destructor body) with ECX=this; if (arg & 1) != 0 calls _free(this); returns this. |
