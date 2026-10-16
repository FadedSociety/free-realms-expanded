# Class: SoeUtil::HashListMap<int,CharacterStat,71,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a10e40` | `HashListMapIntCharacterStat71_Ctor` | ✅ | ctor | Constructs the 71-bucket hash-list map: stores the class vtable, zeroes +8/+0xc/+0x10, memsets the 0x11c-byte bucket array at +0x14 (_memset) and zeroes +4. |
| `00a13490` | `HashListMap_CharacterStat_AllocatorSlot1` | ✅ | virtual | Virtual slot 1 of the map: calls FUN_006fb9c0 then FUN_006fb860 to obtain a memory manager and forwards the request (size 0x1c, alignment 4) to that manager's virtual slot 1, falling back to the global manager at 01bdf2… |
| `00a134d0` | `HashListMap_CharacterStat_AllocatorSlot2_Free` | ✅ | virtual | Eighteen-byte deallocate hook: loads the buffer argument from the stack, pushes alignment 4 and calls the free router FUN_00401c00(ptr,4), then returns with ret 4. |
| `00a163c0` | `HashListMap_CharacterStat_DtorThunk` | ✅ | dtor | Eleven-byte thunk (bytes c701582b8101 e915faffff): writes the class vftable into this and tail-jumps into the clear routine 00a15de0; no frame of its own. |
| `00a164b0` | `HashListMap_CharacterStat_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0): installs the class vftable, calls the clear routine 00a15de0 and frees the object with _free when bit 0 of the flag argument is set. |
| `00a16d00` | `HashListMap_CharacterStat_DtorWithEH` | ✅ | dtor | Non-deleting destructor with an exception frame: installs the class vftable, stores the object in the EH local and calls the clear routine 00a15de0. |
| `00a187c0` | `HashListMap_CharacterStat_ScalarDeletingDtorWithEH` | ✅ | dtor | Scalar deleting destructor with an exception frame: installs the class vftable, calls the clear routine 00a15de0 and frees the object with _free when bit 0 of the flag argument is set. |
