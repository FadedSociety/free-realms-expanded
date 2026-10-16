# Class: Deep::FontPrimitive

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d02d30` | `— ⚠reject` | ⚪ | ctor | Stores the Deep::FontPrimitive vtable at offset 0, owner pointer at +0x4, stores arguments at +0x8..+0x1c (byte at +0x14), and copies the owner's +0x4 pointer's field into +0x2c. |
| `00d03a90` | `FontPrimitiveBuildGlyphQuads` | ✅ | virtual | Virtual slot [0] of Deep::FontPrimitive (not a destructor): reads a UTF-16 string from the object at param_1+0xc via 00701290 (length 1..0x1ff), maps each code unit to a glyph record with 00cfff60 (space 0x20 fallback),… |
