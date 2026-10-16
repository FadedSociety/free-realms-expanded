# Class: std::length_error

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00422756` | `std::length_error::length_error` | ✅ | ctor | Constructor of std::length_error: initialises the logic_error base through FUN_00422736 with the message argument and then overwrites the vtable pointer with std::length_error::vftable. |
| `00422779` | `std::length_error::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor of std::length_error: stores the length_error vtable, runs the logic_error destructor FUN_004202e4 and frees the object with _free when bit 0 of param_2 is set. |
