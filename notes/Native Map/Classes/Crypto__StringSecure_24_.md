# Class: Crypto::StringSecure<24>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b2c880` | `SecureString24Wrapper::Dtor` | ✅ | dtor | Restores the secure wrapper vtable and calls the underlying secure 24-byte string destructor. |
| `00b2cbf0` | `SecureString24Wrapper::ScalarDeletingDtor` | ✅ | virtual | Restores the wrapper vtable, calls its secure string destructor, then conditionally frees the wrapper. |
| `00b30ad0` | `StringSecure24::CtorAssign` | ✅ | ctor | Constructs secure-string base members, installs the Crypto::StringSecure<24> vtable, initializes the secure storage member, and assigns the supplied C string. |
| `00d580d0` | `StringSecure24Ctor` | 🟢 | ctor | Installs IStringFixed<char,24> and Crypto::StringSecure<24> vtables, points the buffer at the shared empty string DAT_01bf2cac with zero length and capacity, then clears the buffer (FUN_00b2ac60) and assigns the C strin… |
