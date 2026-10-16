# Class: SoeUtil::IStringFixed<char,48>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a07040` | `IStringFixed48_Construct ? ⚠rename` | 🟡 | ctor | Constructor of the fixed 48-character string: stores the IString vtable, points the buffer at the shared empty literal, zeroes length and capacity, installs the IStringFixed<char,48> vtable and calls FUN_006f9e60 to ass… |
| `00a07120` | `IStringFixed48_Construct_B` | ✅ | ctor | Second emission of the fixed 48-character string constructor: IString vtable, empty literal, zeroed length/capacity, IStringFixed<char,48> vtable, then assignment from the argument. |
