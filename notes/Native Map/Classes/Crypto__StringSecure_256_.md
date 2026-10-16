# Class: Crypto::StringSecure<256>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008e03c0` | `StringSecure256_Ctor` | ✅ | ctor | Calls 008c9cc0 with this in ECX, writes Crypto::StringSecure<256>::vftable to this, restores the prior exception-list pointer, and returns this. |
| `008e0420` | `StringSecure256_Dtor` | ✅ | dtor | Restores the StringSecure<256> vtable, invokes secure base cleanup, and returns with the prior exception-list state. |
| `008e1f30` | `StringSecure256_ScalarDeletingDtor` | ✅ | dtor | Installs the StringSecure<256> vtable, invokes secure base cleanup, then optionally frees the object when flag bit 0 is set. |
