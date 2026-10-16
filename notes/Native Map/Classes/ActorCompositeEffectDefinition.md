# Class: ActorCompositeEffectDefinition

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00914a10` | `DtorBody → ActorCompositeEffectDefinitionDtorBody` | ✅ | dtor | __fastcall (this=ECX). Under an SEH frame: stores ActorCompositeEffectDefinition::vftable (01804f24) at [this]; stores SoeUtil::List<ActorCompositeEffectEntry,-1> vftable 01804e54 at [this+0x30]; then clears the list wh… |
| `00914aa0` | `VirtualTailJumpSlot2 ? → ActorCompositeEffectDefinition::VirtualTailJumpSlot2` | 🟡 | virtual | Virtual at vftable[3] of 01804f24 (bytes 8b 01 8b 50 08 ff e2): loads the object's vtable, reads slot [2] (+8) and jumps to it with the same stack. It does not return by itself; it tail-dispatches to the virtual at slot… |
| `00914ac0` | `CountAt3cPositive ?` | 🟡 | virtual | Virtual at vftable[2] of 01804f24 (bytes 33 c0 39 41 3c 0f 9f c0 c3): returns 1 when the dword at [this+0x3c] is greater than 0 (signed), else 0. |
| `00914ad0` | `ScalarDeletingDtor → ActorCompositeEffectDefinitionScalarDeletingDtor` | ✅ | dtor | Virtual at vftable[0] of 01804f24 (thiscall, one byte argument). Calls FUN_00914a10 (the destructor body) with ECX=this; if (arg & 1) != 0 calls _free(this); returns this. |
| `00941210` | `ActorCompositeEffectDefinitionCtor` | ✅ | ctor | Constructs its base, installs the definition vtable, initializes the embedded entry list, and zeros its count and head/tail fields. |
| `009412a0` | `ActorCompositeEffectDefinitionInitWithValue` | 🟢 | function | Calls the definition constructor, stores param_2's first word at this+0x48, and returns this. |
