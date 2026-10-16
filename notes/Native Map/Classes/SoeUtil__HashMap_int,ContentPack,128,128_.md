# Class: SoeUtil::HashMap<int,ContentPack,128,128>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01046f00` | `HashMap128_128_DtorBody` | ✅ | dtor | Fastcall, ECX = this. Stores SoeUtil::HashMap<int,ContentPack,128,128>::vftable (0187f450) at *this, calls 01046b70 (clear the 128-bucket nodes), reads the pointer at this+0x20c, zeroes this+0x214 and +0x218, runs 01046… |
| `01047920` | `HashMap128_128::AdjustorThunk_020c_Alloc32` | ✅ | virtual | Thunk. Bytes 81 c1 0c 02 00 00 e9 45 ff ff ff = add ecx,0x20c; jmp 01047870. Adds 0x20c to this and jumps to BlockPool_Alloc32 (01047870); it does no work of its own. |
| `01047930` | `HashMap128_128::AdjustorThunk_020c_Free32` | ✅ | virtual | Thunk. Bytes 81 c1 0c 02 00 00 e9 85 ff ff ff = add ecx,0x20c; jmp 010478c0. Adds 0x20c to this and jumps to BlockPool_Free32 (010478c0); no work of its own. |
| `01047940` | `HashMap128_128::ScalarDeletingDtor` | ✅ | dtor | Thiscall with a flag byte param_2. Calls 01046f00 (the 128/128 destructor body), then if bit 0 of the flag is set calls _free on the object. Returns the object pointer. |
| `01047960` | `— ⚠reject` | ⚪ | ctor | Fastcall, ECX = this. Stores SoeUtil::HashMap<int,ContentPack,128,-1>::vftable at *this, zeroes this+8, memsets 0x200 bytes at this+0xc (128 bucket heads), zeroes this+4, stores SoeUtil::HashMap<int,ContentPack,128,128>… |
