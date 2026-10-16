# Class: GPalette

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0047b8b1` | `GPaletteCtor` | 🟢 | ctor | Constructs a palette object: runs the base initialiser FUN_004d99d1, zeroes the colour-order field at +0x14, the 16-bit field at +0x1a, the colour-data pointer at +0x1c, and installs the two class vtables 01768b18/01768… |
| `0047b90d` | `GPaletteCtorFromData` | 🟢 | ctor | Second palette constructor: same field initialisation as 0047b8b1 (vtables 01768b18/01768b10, +0x14/+0x1a/+0x1c zeroed) and then initialises the palette from param_2 through FUN_004d9a26(param_2,0). |
| `0047c1fe` | `RegisterGPaletteClass` | ✅ | init | Registers the class name GPalette in the engine class registry by calling FUN_0053da53 with the name and the constant 0x18, then publishing through FUN_004ccf27 and FUN_00536678. |
| `0047c231` | `— ⚠reject` | ⚪ | method | Returns 0 when FUN_004c6e70 fails. Otherwise reads the Flags property into +0x14 and the 16-bit Size property into +0x1a. Only when the buffer pointer at +0x1c is nonzero does it look up ColorData; a missing reader or a… |
| `0047c2fb` | `GPaletteDataSize` | 🟢 | method | Calls FUN_004c6e81, whose body is a stub returning 0, zero-extends the u16 values at +0x1a and +0x18, multiplies them, adds the helper result, and returns the sum. In this binary the result is the product of those two f… |
