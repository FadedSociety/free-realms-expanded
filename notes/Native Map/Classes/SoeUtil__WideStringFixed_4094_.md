# Class: SoeUtil::WideStringFixed<4094>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a9b0d0` | `SoeUtil_WideStringFixed4094_FreeBuffer` | 🟢 | method | Frees the pointer argument through 00401c00 with alignment 0 unless it equals the inline buffer at this+0x10, in which case it does nothing. |
| `00a9c600` | `SoeUtil_WideStringFixed4094_Construct` | ✅ | ctor | Calls 00a9b040 (IStringFixed<unsigned_short,4094> constructor) on this and then stores the SoeUtil::WideStringFixed<4094> vtable. |
| `00a9c660` | `SoeUtil_WideStringFixed4094_ScalarDeletingDtor` | ✅ | dtor | Calls 00a9bc80, the shared destructor body of the fixed-capacity UTF-16 strings, on this, then frees the object via _free (00d82463) when bit 0 of the flag byte is set; returns the object pointer. Occupies slot [0] of S… |
