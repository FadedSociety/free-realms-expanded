# Class: SoeUtil::WideStringFixed<16>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0099de70` | `IStringFixed16FreeBuffer` | ✅ | virtual | Compares the supplied buffer with the inline buffer at this+0x10 and frees it only when the pointers differ. |
| `0099f540` | `WideStringFixed16Ctor` | ✅ | ctor | Calls 0099DDE0 with incoming this in ECX to initialize the offset-zero IStringFixed<16> base, then installs the WideStringFixed<16> vtable. |
| `0099f620` | `WideStringFixed16::ScalarDeletingDtor` | ✅ | dtor | Calls 0099DE90 on this in ECX to destroy the offset-zero fixed-string base; if deleting flag bit 0 is set, frees this object, otherwise returns this. |
