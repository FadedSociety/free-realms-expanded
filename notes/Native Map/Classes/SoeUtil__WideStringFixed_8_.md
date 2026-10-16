# Class: SoeUtil::WideStringFixed<8>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0099e070` | `IStringFixed8FreeBuffer` | ✅ | virtual | Checks whether the supplied buffer is the inline buffer at this+0x10 and frees only a different pointer. |
| `0099f5a0` | `WideStringFixed8Ctor` | ✅ | ctor | Calls 0099DFE0 with incoming this in ECX to initialize the offset-zero IStringFixed<8> base, then installs the WideStringFixed<8> vtable. |
| `0099f680` | `WideStringFixed8::ScalarDeletingDtor` | ✅ | dtor | Calls 0099E090 on this in ECX to destroy the offset-zero fixed-string base; if deleting flag bit 0 is set, frees this object, otherwise returns this. |
