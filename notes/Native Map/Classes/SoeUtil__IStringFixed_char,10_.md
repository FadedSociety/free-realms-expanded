# Class: SoeUtil::IStringFixed<char,10>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a55a10` | `— ⚠reject` | ⚪ | dtor | Drops the fixed-string buffer reference count, conditionally frees storage through its vtable, resets inline data, then releases the base IString buffer reference. |
| `00a564b0` | `SoeUtil::IStringFixed10_ScalarDeletingDtor` | ✅ | dtor | Calls the fixed-string destructor and frees the object only if flag bit 0 is set. |
| `00a59380` | `StringFixed10Ctor` | ✅ | ctor | Installs the IString and fixed-string vtables, clears fields, then invokes the string member initializer. |
| `00c68f30` | `SoeUtil::IStringFixed10Ctor` | ✅ | ctor | Constructor body for the fixed 10-character string: stores SoeUtil::IString<char>::vftable at +0, then SoeUtil::IStringFixed<char,10>::vftable at +0, points +4 at the static empty buffer DAT_01bee7b4 and zeroes the leng… |
