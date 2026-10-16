# Class: SoeUtil::Array<SoeUtil::IString<unsigned_short>*,4,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a9ae30` | `SoeUtil_ArrayIStringU16Ptr4_Dtor` | ✅ | dtor | Stores the Array<IString<unsigned_short>*,4,1> vtable, zeroes the count at +8, releases the buffer at +4 through 00401c00 unless it is the inline storage at this+0x13 rounded to 4, then stores the Array<...,0,1> vtable,… |
| `00a9b130` | `SoeUtil_ArrayIStringU16Ptr4_ScalarDeletingDtor` | 🟢 | dtor | Calls 00a9ae30 (the Array<IString<unsigned_short>*,4,1> destructor body) on this, then if the low bit of the flag byte is set calls _free on the object and returns it. |
| `00a9b1b0` | `SoeUtil_ArrayIStringU16Ptr4_Construct` | ✅ | ctor | Stores the Array<IString<unsigned_short>*,0,1> vtable, zeroes the words at +4, +8 and +0xc, then stores the Array<IString<unsigned_short>*,4,1> vtable. |
