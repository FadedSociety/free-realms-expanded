# Class: Crypto::IStringSecure<char,256>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008c9cc0` | `Crypto_IStringSecure256::Ctor` | ✅ | ctor | Installs an SEH frame, stores Crypto::IStringSecure<char,256>::vftable at this+0, adjusts ECX to this+4 and calls FUN_00402090 as the fixed-string base constructor on that subobject, restores the prior exception chain, … |
| `008e0360` | `IStringSecure256_Cleanup` | ✅ | dtor | Stores the Crypto secure-string vtable, runs fixed-256 string cleanup, and restores the prior exception-list pointer. |
| `008e1f10` | `StringSecure256_ScalarDeletingDtor` | ✅ | dtor | Calls secure IString cleanup and frees the object when flag bit 0 is set; otherwise returns the object pointer. |
