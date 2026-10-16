# Class: GStringObject

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00428da5` | `GStringObject::GStringObject` | ✅ | ctor | Constructor of the GStringObject class: runs the base constructor 004d99d1, installs GStringAccess at +0x14 and GDataAccessImpl at +0x18, zeroes +0x20/+0x24/+0x28, stores param_3 at +0x2C, overwrites the four interface … |
| `0042c58a` | `GStringObject::QuerySlot74WithTemp` | ✅ | virtual | Second emission of the same body as QuerySlot74WithTempA: FUN_00428293(1, param_2), then the receiver's vtable slot +0x74 with a 16-byte stack result, then FUN_004220f2 cleanup; returns the slot's boolean result. |
| `0044d239` | `GStringObject::GStringObject` | ✅ | ctor | Constructs a GStringObject around a wide-character source: runs the base constructor 004d99d1, installs the GStringAccess::vftable and GDataAccessImpl::vftable subobject pointers at +0x14/+0x18, zeroes +0x20..+0x28, sto… |
| `0044d294` | `GStringObject::GStringObject_2` | ✅ | ctor | Ansi constructor of the same GStringObject layout: base constructor 004d99d1, the two interface vtables at +0x14/+0x18, zeroed fields, param_3 at +0x2c, GStringObject::vftable in the four headings, +0x30 pointing at DAT… |
