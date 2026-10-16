# Class: FlashEntity

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d55730` | `FlashEntity::DestroyViaVtable ?` | 🟡 | virtual | Calls FUN_0075c060 first, then if the argument pointer is non-null invokes its vtable slot 0 with flag 1 (scalar deleting destructor, i.e. delete-by-vtable). |
| `00d55c40` | `— ⚠reject` | ⚪ | virtual | Resolves the current frame counter: from the field +0x38 of the object at this+4 when set, else from FUN_00729610 when this+8 is set, else zero; reads its +0x40 value; if it differs from the value cached at (this+0x14)+… |
| `00d561c0` | `— ⚠reject` | ⚪ | ctor | Installs FlashEntity::vftable, stores the argument pointer at this+0x14, registers this in a list through 00d55e00, and when bit 1 of the flags word at this+4 is clear sets it, calls FUN_00735a30 when the word at this+4… |
| `00d56270` | `— ⚠reject` | ⚪ | dtor | Restores FlashEntity::vftable in *param_1, calls ListRemoveValue (00d56140) on this entity's child list, then calls the base teardown FUN_00764e10 with the saved security cookie. |
| `00d56520` | `FlashEntity::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: calls FlashEntity::Dtor (00d56270) on this, then calls _free(this) when bit 0 of the flags argument is set; returns this and pops one stack argument. |
