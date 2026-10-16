# Class: Crypto::StringSecure<8>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b2c930` | `SecureString8Wrapper::Dtor` | ✅ | dtor | Restores the secure wrapper vtable and invokes the underlying secure 8-byte string destructor. |
| `00b2cc70` | `SecureString8Wrapper::ScalarDeletingDtor` | ✅ | virtual | Restores the wrapper vtable, calls the secure string destructor, and conditionally frees the wrapper. |
| `00b30b40` | `StringSecure8::CtorAssign` | ✅ | ctor | Constructs secure-string base members, installs the Crypto::StringSecure<8> vtable, initializes secure storage, and assigns the supplied C string. |
| `00d58150` | `StringSecure8Ctor` | 🟢 | ctor | Installs IStringFixed<char,8> and Crypto::StringSecure<8> vtables, points the buffer at the shared empty string DAT_01bf2cac with zero length and capacity, then clears the buffer (FUN_00b2ad40) and assigns the C string … |
