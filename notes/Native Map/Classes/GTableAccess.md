# Class: GTableAccess

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0066347d` | `— ⚠reject` | ⚪ | virtual | Stack argument code (param_2) mapped to a dword: 4 gives 0x14 via a keyed read (DAT_01bdeaac through vtbl+0x18 of the sub-object at +0x24, written into the local); 0 gives 3; 3 gives 0x32; 0xc gives 0x1000; 0xd or 0xe g… |
| `006634e4` | `GTableAccess::InitCellRecordByType` | 🟢 | virtual | thiscall(this, param_2, param_3 type code, param_4, param_5 out record). Returns 0 when param_5 is null or param_3 is neither 0 nor 2. For type 0 sets *param_5 = 0xd and releases param_5[2] when non-null, result 3; for … |
| `00663588` | `GTableAccess::ForwardKeyedToSlot110 ?` | 🟡 | virtual | Reads a held object from this+0x24 through its vtbl+0x18 with DAT_01bd9f14 (a 16-bit key). When that object is non-null and its vtbl+0x110(param_2, this, param_3, param_4, param_5) returns nonzero returns 1; otherwise r… |
| `006635d3` | `GTableAccess::KeyedForwardToSlot10C ?` | 🟡 | virtual | Zeroes the two dwords at param_1 (the output), reads a held object from this+0x24 through its vtbl+0x18 with DAT_01bd9f14, and when non-null calls its vtbl+0x10c(out, this, param_3, param_4). Returns the output pointer. |
