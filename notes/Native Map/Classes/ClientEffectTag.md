# Class: ClientEffectTag

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a0fcd0` | `ClientEffectTag_Ctor` | 🟢 | ctor | Constructs a ClientEffectTag: runs the EffectTag base constructor FUN_0103daa0, stores ClientEffectTag::vftable, then zeroes the tag payload fields at +0x68, the byte at +0x6c and the dwords at +0x70 and +0x74. |
| `00a0fd30` | `ClientEffectTag_Dtor` | 🟢 | dtor | Destructor body: stores ClientEffectTag::vftable back into this and calls the EffectTag base destructor FUN_0103da90 (registered as an unwind funclet target for several EH ranges). |
| `00a0fd80` | `ClientEffectTag_ScalarDeletingDtor` | 🟢 | dtor | Scalar deleting destructor: restores ClientEffectTag::vftable, calls the base EffectTag destructor FUN_0103da90 and, when bit 0 of param_2 is set, frees the object with _free. |
| `00a10f50` | `ClientEffectTag_Dtor2` | 🟢 | dtor | Second destructor body for ClientEffectTag: restores ClientEffectTag::vftable and calls the EffectTag base destructor FUN_0103da90; no fields released. |
| `00a10fa0` | `ClientEffectTagDerived_Ctor` | 🟢 | ctor | Constructs a ClientEffectTag-derived object: calls the ClientEffectTag constructor FUN_00a0fcd0 with this, then copies *param_2 into the field at this+0x80 and returns this. |
| `00a13690` | `ClientEffectTag_ScalarDeletingDtor` | ✅ | dtor | Stores ClientEffectTag::vftable, calls FUN_0103da90 with this object in ECX (the EffectTag base destructor installs EffectTag::vftable), then calls _free on the object only when flag bit 0 is set. |
