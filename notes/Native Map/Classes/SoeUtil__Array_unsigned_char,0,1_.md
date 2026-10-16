# Class: SoeUtil::Array<unsigned_char,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00898200` | `SoeUtil::ArrayByteDtor` | ✅ | dtor | Destructor body of SoeUtil::Array<unsigned_char,0,1>: installs the Array vtable at +0, releases the owned buffer at +4 with FreeMemory(...,1), then clears the count at +8 and the buffer pointer at +4. |
| `008983c0` | `ArrayByteScalarDtor` | ✅ | dtor | Scalar deleting destructor of SoeUtil::Array<unsigned_char,0,1>: installs the Array vtable, releases the buffer at +4 with FreeMemory(...,1), clears fields and calls _free on the object when bit 0 of param_2 is set. |
