# Class: GBitmap

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0047bcdb` | `RegisterGBitmapClass` | ✅ | init | Registers the class name GBitmap in the engine class registry by calling FUN_0053da53 with the name and the constant 0x18, then publishing through FUN_004ccf27 and FUN_00536678. |
| `004860fc` | `GBitmap::CopyFrom` | 🟢 | method | Copy/assign routine of the GBitmap class (its caller 004945D3 is the GBitmap class-registry record factory, whose create op allocates 0x30 bytes and whose copy op calls this function): copies the nine leading dwords via… |
| `0048612f` | `GBitmap::~GBitmap` | 🟢 | dtor | Destructor of the GBitmap class: releases the interface at +0x2C through vtable slot +0x10 (zeroing the field, then a now-dead second guarded release) and frees the object with _free when the low bit of param_2 is set; … |
