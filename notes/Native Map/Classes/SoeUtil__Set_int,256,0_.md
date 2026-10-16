# Class: SoeUtil::Set<int,256,0>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bd490` | `SetInt256::InitStorage` | 🟢 | init | Zeroes list state, links 0x40 inline nodes while advancing a 32-bit pointer by 0x14 elements (0x50 bytes) per iteration, then stores capacity 0x100. |
| `00a3c400` | `Set256_Construct → Set256SubobjectAt0C_Construct` | ✅ | ctor | Constructor of the 256-entry SoeUtil::Set: installs the base Set<int,-1,0> vtable, zeroes the two table fields at +4/+8 under an SEH frame, then installs Set<int,256,0>::vftable and calls FUN_008bd490 (the table constru… |
| `00a3c480` | `— ⚠reject` | ⚪ | dtor | Destructor body of the 256-entry set: installs Set<int,256,0>::vftable, frees the first table's two subtrees with FUN_007a2470 and then frees the root through the set's virtual slot +0xc, clears the table/count fields, … |
| `00a3c580` | `Set256_Slot2Adjustor_0C` | ✅ | virtual | Adds 0x0c to ECX, then jumps to 008f37a0; this is an adjustor thunk, so the target body is not executed here. |
| `00a3c590` | `Set256_Slot3Adjustor_0C` | ✅ | virtual | Adds 0x0c to ECX, then jumps to 008f3810; this is an adjustor thunk, so the target body is not executed here. |
| `00a3c5a0` | `Set256_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor of the 256-entry set: calls the teardown body 00a3c480 and then calls _free(this) when bit 0 of the flag argument is set; returns this otherwise. |
