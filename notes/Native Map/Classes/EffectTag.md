# Class: EffectTag

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bf990` | `EffectTag::ScalarDeletingDestructor` | 🟢 | dtor | Calls 0x0103da90 with this in ECX and no stack arguments; the callee installs EffectTag::vftable and returns. Frees this when deleting-flag bit 0 is set. |
| `009dc030` | `EffectTag::EffectTag` | ✅ | ctor | Copy constructor for EffectTag: stores EffectTag::vftable and copies the source fields at param_2+8..+0x48, +0x50..+0x5c and the three bytes at +0x60..+0x62 into the new object. |
| `009dc0d0` | `EffectTag::ScalarDeletingDtor` | 🟢 | dtor | Scalar deleting destructor: calls the destructor body FUN_0103da90 and frees the object when (param_2 and 1) is set. Slot 0 of EffectTag::vftable (dword at 0x01810578 is this address). |
