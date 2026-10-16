# Class: SoeUtil::IStringFixed<unsigned_short,4094>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a9b040` | `SoeUtil_IStringFixedU16Init` | ✅ | ctor | Stores SoeUtil::IString<unsigned_short>::vftable, sets the buffer to the empty-string constant DAT_01be7ac0 and zeroes the length and capacity words, then stores SoeUtil::IStringFixed<unsigned_short,4094>::vftable. |
| `00a9bc80` | `SoeUtil_IStringFixedU16_4094_Dtor` | ✅ | dtor | Stores the IStringFixed<unsigned_short,4094> vtable, releases the string buffer at +4 when the count at +0xc is positive (interlocked decrement and vtable slot +8 at the last reference), resets the buffer to DAT_01be7ac… |
| `00a9bd40` | `SoeUtil_IStringFixedU16_4094_ScalarDeletingDtor` | 🟢 | dtor | Calls 00a9bc80 (the IStringFixed<unsigned_short,4094> destructor body) on this, then if the low bit of the flag byte is set calls _free on the object and returns it. |
