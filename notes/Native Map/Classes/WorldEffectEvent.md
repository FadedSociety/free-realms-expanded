# Class: WorldEffectEvent

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a2660` | `WorldEffectEvent::~WorldEffectEvent` | ✅ | dtor | Stores WorldEffectEvent::vftable at +0 and EffectEventInterface::vftable at +0x44, calls FUN_007e7980 with this in ECX, then restores the SEH chain. |
| `008a27f0` | `WorldEffectEvent::ScalarDeletingDestructor` | 🟢 | dtor | Thin deleting destructor: calls FUN_008a2660 on this and then calls _free(this) when param_2 bit 0 is set. |
| `008b6130` | `WorldEffectEvent::WorldEffectEvent` | ✅ | ctor | Runs a base initialiser through 007e78e0, stores EffectEventInterface::vftable at +0x44, then WorldEffectEvent::vftable at *this and at +0x44, zeroes the byte at +0x48 and stores param_4 at +0x4c, and finally copies 16 … |
