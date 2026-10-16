# Class: Crypto::ArraySecure<unsigned_char,32,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d2e650` | `WipeAndFreeBuffer` | 🟢 | virtual | Virtual slot 2 of the secure array: when the count at +8 is positive it wipes that many bytes at +4 through 00d2e4d0, then frees the buffer passed as the first stack argument through 00401c00 (alignment 1) unless that p… |
| `00d2e700` | `DtorBody` | ✅ | dtor | Destructor body: stores the ArraySecure vptr, sets the count to 0, frees the heap buffer at +4 through 00401c00 unless it is the inline storage at +0x10, then installs the SoeUtil::Array<unsigned_char,32,1> and SoeUtil:… |
| `00d2e8a0` | `ScalarDeletingDtor` | ✅ | dtor | Virtual slot 0 of ArraySecure: calls the body 00d2e700, then frees this through _free when bit 0 of the argument is set; returns this. |
