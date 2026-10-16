# Class: std::bad_alloc

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00403aad` | `std::bad_alloc::bad_alloc` | ✅ | ctor | Constructs a std::bad_alloc exception object: runs the std::exception base constructor with the caller's argument block and writes the std::bad_alloc vtable at +0; returns this. |
| `00403ad1` | `std::bad_alloc::~bad_alloc` | ✅ | dtor | Deleting destructor for std::bad_alloc: rewrites the std::bad_alloc vtable, calls the base exception cleanup FUN_00d83915, then frees the object with _free when the flag argument's low bit is set. |
