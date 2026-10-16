# Class: Crypto::IStringSecure<char,8>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b2ace0` | `SecureString8::Constructor ?` | 🟡 | ctor | Writes the secure fixed-string vtable and initializes the 8-byte string storage. |
| `00b2c8d0` | `SecureString8::Dtor` | ✅ | dtor | Restores the secure 8-byte string vtable, clears its buffer, and calls member cleanup. |
| `00b2cc50` | `SecureString8::ScalarDeletingDtor` | ✅ | virtual | Calls the secure 8-byte string destructor, conditionally frees the object, and returns its pointer when retained. |
