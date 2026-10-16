# Class: SoeUtil::IStringFixed<char,32>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00703350` | `FixedStringDestructor ?` | 🟡 | dtor | Decrements the shared string reference and resets the fixed-string base to its empty representation. |
| `007033e0` | `FixedStringScalarDeletingDestructor ?` | 🟡 | dtor | Calls the fixed-string destructor and conditionally releases the outer object when the deleting flag is set. |
| `00703480` | `FixedStringDestructorDuplicate ?` | 🟡 | dtor | Releases the shared string reference and restores the fixed-string object's empty base state. |
| `007035c0` | `FixedStringScalarDeletingDestructorDuplicate ? → FixedString32ScalarDeletingDes…` | 🟡 | dtor | Runs fixed-string destruction and conditionally frees the containing object according to the deleting flag. |
| `008d3bf0` | `Ctor_IStringFixed32 ?` | 🟡 | ctor | Constructor of a fixed 32-byte string: initializes the SoeUtil::IString<char> base (vtable, empty buffer 01be48fc, zero length/capacity), overwrites the vtable with SoeUtil::IStringFixed<char,32>::vftable and assigns th… |
| `00d064a0` | `IStringFixed32DtorBody` | ✅ | dtor | Non-deleting destructor body: stores the IStringFixed<char,32> vtable, releases the heap buffer at +0x4 when its count at +0xc is positive (LOCK decrement, destroy through vtable slot 2 on the last reference), resets th… |
| `00d067c0` | `IStringFixed32ScalarDtor` | ✅ | dtor | Scalar deleting destructor: stores the IStringFixed<char,32> vtable, releases the heap buffer at +0x4 when its count at +0xc is positive, resets the buffer to the static empty string (DAT_01bf2604) with zero count and l… |
