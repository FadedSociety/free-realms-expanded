# Class: SoeUtil::Array<SoeUtil::IString<unsigned short>*,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a9a710` | `SoeUtil_ArrayIStringU16Ptr_AllocBuffer` | 🟢 | method | Buffer-policy method for an array of pointers: when the flag argument is zero and the requested count exceeds the capacity at this+0xc it grows to requested*5/4, when the capacity is within requested..requested*4/3 it r… |
| `00a9a7b0` | `SoeUtil_ArrayIStringU16Ptr_FreeBuffer` | 🟢 | method | Forwards the pointer argument to 00401c00 (FreeMemory) with alignment 4 and returns. |
| `00a9a7d0` | `SoeUtil_ArrayIStringU16Ptr_Swap` | 🟢 | method | Thiscall with a second array and a flag. When the flag is nonzero it tail-delegates to the argument array's vtable slot [3] (register arguments not recovered). Otherwise it exchanges the buffer pointer at this+4, the el… |
