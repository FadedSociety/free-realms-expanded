# Class: GBitmapAccessWrapper

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0041507f` | `GBitmapAccessWrapper::ForwardAccessSlot78` | 🟢 | virtual | When the data access object at +8 is present, tail-calls its vtable slot +0x78 with no arguments and returns the result; otherwise returns 0. |
| `0041508e` | `GBitmapAccessWrapper::ForwardAccessSlot90` | 🟢 | virtual | Returns 0 when param_2 is null or no data access is held at +8; otherwise forwards param_2 to the access object's vtable slot +0x90 and returns its result. |
| `00420e36` | `GBitmapAccessWrapper::Ops40` | 🟢 | function | Opcode-dispatched construction of 40-byte GBitmapAccessWrapper objects with the same shape as 00420db2: ops 2/3 allocate 0x28 bytes with FUN_00d833a6, ops 0..3 initialise through FUN_0041493a (ops 1 and 3 additionally p… |
| `0042afb6` | `GBitmapAccessWrapper::GBitmapAccessWrapper` | ✅ | ctor | Constructor of the GBitmapAccessWrapper class: runs the base initializer 0041493a, installs the GBitmapAccessWrapper vtable at +0, allocates 0x34 bytes through 0047d9ec and constructs the bitmap data-access object via 0… |
