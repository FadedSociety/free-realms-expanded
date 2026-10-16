# Class: SoeUtil::IString<unsigned_short>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a99760` | `SoeUtil_IStringU16_Construct` | ✅ | ctor | Returns 0 when given null. Otherwise stores SoeUtil::IString<unsigned_short>::vftable, sets the buffer pointer to the empty-string constant DAT_01be7ac0 and zeroes the length and capacity words. |
| `00be8120` | `— ⚠reject` | ⚪ | dtor | Stores the SoeUtil::IString<unsigned_short> vtable at this+4, and when the length field at this+0x10 is positive decrements the reference count at buffer-4 with a LOCK'd exchange and, if the result was 1 (or negative), … |
| `00be8340` | `IStringUnsignedShort_Ctor` | 🟢 | ctor | Copies the dword at *param_2 into this+0, stores the SoeUtil::IString<unsigned_short> vtable at this+4, stores the address of the shared empty-buffer sentinel DAT_01bead04 at this+8, zeroes this+0xc and this+0x10, then … |
| `00c8f610` | `IStringWideDtor` | ✅ | dtor | Restores the wide IString vtable, decrements its buffer reference count, and releases the buffer when the count reaches zero. |
| `00c8f6d0` | `IStringWideScalarDeletingDtor` | 🟢 | dtor | Destroys the wide string state and frees the object when deleting flag bit 0 is set. |
| `00c8f7c0` | `IStringWideCtor → ProfileSortingData::Ctor` | ✅ | ctor | Stores supplied object fields, installs the wide IString vtable and empty buffer, clears reference counters, and initializes its string contents. |
