# Class: SoeUtil::Array<LeaderBoardResponse::EntryLine,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b857e0` | `LeaderBoardEntryArray_Reserve` | 🟢 | virtual | Buffer acquisition for a 0x70-byte-element array whose vtable is SoeUtil::Array<LeaderBoardResponse::EntryLine,0,1>::vftable (slot 1): reads the capacity at this+0xc; without the force flag it grows the request to 5/4 (… |
| `00b85880` | `LeaderBoardEntryArray_Free` | 🟢 | virtual | Deallocation hook (slot 2) of the entry array: calls FUN_00401c00(param_1,8) i.e. FreeMemory with 8-byte alignment on the passed buffer pointer and returns. |
| `00b858a0` | `— ⚠reject` | ⚪ | virtual | Swap/move hook (slot 3) of the entry array: with param_3 set it tail-forwards through the other array's slot 3 (virtual call at *param_2+0xc) and returns; otherwise it swaps this+4/param_2[1] (buffer), this+8/param_2[2]… |
| `00b860d0` | `— ⚠reject` | ⚪ | method | Resizes the entry array: calls the array vtable slot 1 (offset +4) to reserve a new element count, then if the returned buffer differs from the old one (this+4) it moves every old element up to the old count (this+8) wi… |
| `00b86200` | `LeaderBoardEntryArray_RemoveTail` | 🟢 | method | Destroys entries from the tail: newCount = count(this+8) - param_2, destroys each element at index newCount..oldCount-1 by calling FUN_004022b0 on the fixed-string member at element+8 (element stride 0x70) and writes th… |
| `00b86300` | `— ⚠reject` | ⚪ | method | Appends a copy of the entry supplied in the argument: if count(this+8)+1 exceeds the capacity at this+0xc it calls the resize helper 00b860d0, then constructs the new element at buffer(this+4) + count*0x70 from the argu… |
| `00b86440` | `LeaderBoardEntryArray_Destroy` | 🟢 | dtor | Destructor body of the entry array: resets the vtable, destroys the elements from the count at this+8 via 00b86200, frees the buffer at this+4 with alignment 8 through FUN_00401c00 and zeroes this+4. |
| `00b86470` | `LeaderBoardEntryArray_ScalarDeletingDtor` | 🟢 | dtor | Vtable slot 0 scalar deleting destructor of the entry array: resets the vtable, destroys the elements through 00b86200, frees the buffer at this+4 with alignment 8, zeroes this+4 and frees the array object itself when b… |
