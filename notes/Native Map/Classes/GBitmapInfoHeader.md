# Class: GBitmapInfoHeader

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0047bb91` | `RegisterGBitmapInfoHeaderClass` | ✅ | init | Registers the class name GBitmapInfoHeader in the engine class registry by calling FUN_0053da53 with the name and the constant 0x18, then publishing through FUN_004ccf27 and FUN_00536678. |
| `0047bbc4` | `CopyGBitmapInfoHeader` | 🟢 | function | Copies the 9-dword GBitmapInfoHeader structure field by field from param_2 to param_1. |
