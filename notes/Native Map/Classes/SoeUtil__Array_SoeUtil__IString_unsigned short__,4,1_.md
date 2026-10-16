# Class: SoeUtil::Array<SoeUtil::IString<unsigned short>*,4,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a9a840` | `SoeUtil_ArrayIStringU16Ptr4_FreeBuffer` | 🟢 | method | Frees the pointer argument through 00401c00 with alignment 4 unless it equals the object's inline storage address this+0x13 rounded down to 4 (this+0x10), in which case it does nothing. |
| `00a9aef0` | `SoeUtil_ArrayIStringU16Ptr4_AllocBuffer` | 🟢 | method | Buffer-policy method for an array with four inline pointer slots: when the requested count is above 4 it delegates to 00a9a710 (the same policy without inline storage) and returns its buffer; otherwise it reports capaci… |
| `00a9c6c0` | `SoeUtil_ArrayIStringU16Ptr4_Swap` | 🟢 | method | Thiscall with a second array of the same class. Builds a scratch Array<SoeUtil::IString<unsigned_short>*,0,1> in the stack frame, appends the argument array's elements to it through 00a9b270 and zeroes the argument's co… |
