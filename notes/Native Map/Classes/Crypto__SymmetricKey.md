# Class: Crypto::SymmetricKey

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d2e480` | `HasNonZeroKeyByte` | 🟢 | virtual | Virtual slot 3 of SymmetricKey: returns AL=1 when the count at this+0xC is positive and at least one byte of the buffer at this+8 is nonzero; otherwise AL=0. |
| `00d2e760` | `Ctor` | ✅ | ctor | Default constructor: stores the SymmetricKey vptr at +0, zeroes the dwords at +8, +0xC and +0x10, stores the embedded ArraySecure vptr at +4 and returns this in EAX. |
| `00d2e780` | `WipeAndClear` | 🟢 | virtual | Virtual slot 2 of SymmetricKey. Passes the embedded array (this+4) as receiver: when its count is positive it wipes count bytes through 00d2e4d0; when count and capacity are both negative it calls 00701cc0(0,1); then se… |
| `00d2e7c0` | `CtorFromBytes` | ✅ | ctor | Constructs a SymmetricKey: stores the SymmetricKey and ArraySecure vptrs, zeroes the array fields, calls its own virtual slot 2 (WipeAndClear), then assigns the (param_2, param_3) bytes into the embedded array through 0… |
| `00d2e840` | `DtorBody` | ✅ | dtor | Destructor body: stores the SymmetricKey vptr, adds 4 to ECX and tail-jumps to 00d2e700 (the embedded ArraySecure destructor body). |
| `00d2e8c0` | `ScalarDeletingDtor` | ✅ | dtor | Virtual slot 0 of SymmetricKey: stores the SymmetricKey vptr, calls the array body 00d2e700 on this+4, then frees this through _free when bit 0 of the argument is set; returns this. |
