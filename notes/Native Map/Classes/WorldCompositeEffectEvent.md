# Class: WorldCompositeEffectEvent

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008c51a0` | `WorldCompositeEffectEvent::Constructor` | ✅ | ctor | Calls WorldEffectEvent initialization, installs WorldCompositeEffectEvent vtables at offsets 0 and +0x44, stores param_2 at +0x90, clears +0x94, and copies param_3 into two byte fields. |
| `008c5240` | `WorldCompositeEffectEvent::GetResult` | ✅ | virtual | Calls the secondary object at this+0x44 virtual slot 1; if nonnull, calls that result object slot 4 and returns its value, otherwise returns zero. |
| `008c5270` | `WorldCompositeEffectEvent::HasUnsetValue` | ✅ | virtual | Calls the secondary object at this+0x44 virtual slot 1 and returns true only when the returned object field at +0x1c equals -1. |
| `008c5290` | `WorldCompositeEffectEvent::AdjustorThunk` | ✅ | virtual | Adjusts ECX by -0x44 and jumps to 0x008c5500; this is an adjustor thunk, not the callee body. |
| `008c5380` | `WorldCompositeEffectEvent::~WorldCompositeEffectEvent` | ✅ | dtor | Stores WorldCompositeEffectEvent::vftable at this+0 and this+0x44 (param_1[0x11]). If this+0x94 is nonzero, calls FUN_00769710 and, when its +0x16c field is nonzero, calls FUN_007dc240. Then calls the base destructor FU… |
| `008c5410` | `WorldCompositeEffectEvent::VSlot12` | ✅ | virtual | If this+0x94 is nonzero, calls FUN_00769710 and, when its +0x16c field is nonzero, calls FUN_007dc1e0 on that sub-object with a float 0.0 and int 2 pushed as stack args (ret 8). Otherwise returns. |
| `008c5450` | `WorldCompositeEffectEvent::VSlot8_CopyBlock64AndNotify` | 🟢 | virtual | Copies 16 dwords (64 bytes) from param_2 into this+0x50. If this+0x94 is nonzero and FUN_00769710()+0x16c is nonzero, runs NodeTable_FindKey on key this+0x94 (or retries it), then walks matching nodes: calls each node v… |
| `008c5500` | `WorldCompositeEffectEvent::ScalarDeletingDtor` | ✅ | dtor | Calls the destructor 008c5380 on this, then frees the block with _free when bit 0 of the stack flag byte is set. Returns this (ret 4). |
| `008d1200` | `WorldCompositeEffectEvent::Slot1_ManagerLookup` | 🟢 | virtual | Virtual slot [1] override for the world composite effect event: calls the client-singleton accessor FUN_00769710, loads the key at this+0x4c, fetches the manager pointer at singleton+0xac through FUN_00769ab0 and runs t… |
