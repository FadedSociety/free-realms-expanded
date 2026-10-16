# Class: Crypto::IStringSecure<char,24>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b2ac00` | `SecureString24::Constructor` | ✅ | ctor | Writes the secure fixed-string vtable and initializes 24-byte-inline string storage. |
| `00b2c820` | `SecureString24::Dtor` | ✅ | dtor | Restores the secure string vtable, clears its buffer, and calls the string base cleanup routine. |
| `00b2cbd0` | `SecureString24::ScalarDeletingDtor` | ✅ | virtual | Calls the secure string destructor, conditionally frees the object, and returns the pointer if not deleting. |
