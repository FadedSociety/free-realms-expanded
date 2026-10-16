# Class: SoeUtil::Array<Target,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bdbd0` | `TargetArray::Allocate` | ✅ | virtual | Computes a capacity, returns existing storage when reusable, or allocates capacity times 8 bytes aligned to 4. |
| `008bdc70` | `TargetArray::FreeBuffer` | ✅ | virtual | Releases the supplied array buffer with 4-byte alignment. |
| `008bdc90` | `TargetArray::DispatchOrSwap` | ✅ | virtual | Dispatches the supplied entry callback when flagged; otherwise swaps the entry words at +4, +8 and +0xc. |
| `008c7b70` | `— ⚠reject` | ⚪ | dtor | Stores SoeUtil::Array<Target,0,1>::vftable at this+0, calls FUN_008bedb0 (unlabelled, no arguments), frees the buffer at this+4 through FUN_00401c00 with alignment 4, and zeroes this+4. Returns void. |
| `008c9370` | `SoeUtil_ArrayTarget_ScalarDeletingDtor` | ✅ | dtor | Stores the Array<Target,0,1> vtable at this+0, calls FUN_008bedb0, frees the buffer at +4 through FUN_00401c00 (alignment 4), and frees this with _free when bit 0 of the flag byte is set. Returns this. |
| `008d40a0` | `CopyCtor_ArrayTarget` | 🟢 | ctor | Copy constructor of the growable Target array: installs SoeUtil::Array<Target,0,1>::vftable and zeroes the element pointer, count and capacity, then when the source array's count at +8 is positive calls FUN_008c9f90 wit… |
