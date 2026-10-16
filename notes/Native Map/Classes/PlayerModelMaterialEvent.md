# Class: PlayerModelMaterialEvent

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0095c800` | `PlayerModelMaterialEvent::PlayerModelMaterialEvent` | 🟢 | ctor | Calls the PlayerEffectEvent base constructor, installs PlayerModelMaterialEvent vtables at this and this+0x50, stores the input object pointer at this+0xa0, increments that object?s +8 count, and calls its virtual slot … |
| `0095c8b0` | `PlayerModelMaterialEvent::~PlayerModelMaterialEvent` | ✅ | dtor | Restores the model-material vtables, decrements the referenced object's count if positive, and calls FUN_00957df0. |
| `0095c920` | `PlayerModelMaterialEvent::GetWordAt50` | 🟢 | virtual | Returns the four-byte value stored at this+0x50. |
| `0095c930` | `PlayerModelMaterialEvent::ScalarDeletingDtor` | ✅ | virtual | Calls FUN_0095c8b0 and frees param_1 when the deleting flag bit is set. |
| `0095c950` | `PlayerModelMaterialEvent::ScalarDeletingDtorAdjustorThunk` | ✅ | virtual | Subtracts 0x50 from ECX, then jumps to 0095c930. |
| `0095f950` | `— ⚠reject` | ⚪ | virtual | Resolves the current player and effect object, reads a value through FUN_00776310, performs cleanup, and returns the value or zero. |
