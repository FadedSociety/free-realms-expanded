# Class: std::logic_error

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `004202e4` | `std::logic_error::~logic_error` | ✅ | dtor | Destructor of std::logic_error: rewrites the object's vtable pointer to std::logic_error::vftable, then runs the string tidy FUN_0041ba81(1,0) on the embedded std::string member and calls the base-class destructor FUN_0… |
| `0042031d` | `std::logic_error::ScalarDeletingDtor` | 🟢 | dtor | MSVC scalar deleting destructor: runs the logic_error destructor FUN_004202e4 on the receiver and, when bit 0 of param_2 is set, releases the object with _free and returns the pointer. |
| `00422736` | `std::logic_error::logic_error` | ✅ | ctor | Constructor of std::logic_error: runs the exception base constructor FUN_00d837f0, stores the std::logic_error vtable into the object and copy-constructs the message string member through the std::string copy constructo… |
