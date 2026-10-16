# Class: GPropertyInterface

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00411de6` | `GPropertyInterface::Ctor` | ✅ | ctor | Constructs an object with three vtable sub-objects: calls FUN_004d99d1, writes GPropertyInterface::vftable into the sub-object at +0x14 (then replaces it with PTR_LAB_0175d73c), writes PTR_FUN_0175d770/PTR_LAB_0175d76c … |
| `00411e80` | `GPropertyInterface::Dtor` | ✅ | dtor | Destructor: releases the interface pointer at +0x18 (virtual slot +0x10), writes GInterface::vftable into the sub-object at +0x14 (bytes: neg/sbb/and select param_1+0x14 or null) and then tail-jumps to the base destruct… |
| `00411f26` | `GPropertyInterface::CtorWithSource` | ✅ | ctor | Construction from an existing source: calls FUN_004e68fb(param_2), writes the same three vtable pointers, AddRefs the interface at param_2+0x18 (virtual +0xc) and stores it at +0x18. |
| `0041a5d4` | `GPropertyInterface::ScalarDeletingDtor` | 🟢 | dtor | Calls FUN_00411E80 on this, frees this through the operator-delete thunk when param_2 bit 0 is set, and returns this otherwise. |
| `0049327c` | `GPropertyInterface::Construct ⚠downgrade` | 🟢 | ctor | If param_1 is non-null, installs GPropertyInterface::vftable and initializes member vtables, the shared empty-string marker at +0x14 and its companion pointer at +0x18, a 16-bit 0xffff field at +0x38, constants 0x100 at… |
| `0049369f` | `GPropertyInterface::ConstructFromSource` | 🟢 | ctor | If param_1 is non-null, installs GPropertyInterface::vftable and initializes the base members. Computes source base as param_2+0x10 when param_2 is non-null (otherwise zero), copies source offsets +0xc..+0x28 into desti… |
