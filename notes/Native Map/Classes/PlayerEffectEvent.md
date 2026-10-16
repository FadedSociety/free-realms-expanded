# Class: PlayerEffectEvent

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00957df0` | `PlayerEffectEventDtor ?` | 🟡 | dtor | Restores PlayerEffectEvent and embedded interface vtables, releases the owned interface pointer when non-null, restores PlayerEvent base vtable, and calls base cleanup. |
| `00957e90` | `PlayerEffectEvent::IsFinished ?` | 🟡 | virtual | Calls the embedded object's virtual slot 1; returns true when absent or when its field at +0x1c equals -1. |
| `00957eb0` | `PlayerEffectEvent::ApplyEmbeddedEffect ?` | 🟡 | virtual | If an embedded object exists and its virtual slot 5 returns true, calls its slots 2 and then the event object's slot 3 with argument one. |
| `00957ef0` | `PlayerEffectEvent::ScalarDeletingDtor` | 🟢 | dtor | Calls the PlayerEffectEvent destructor and frees the object when deletion flag bit 0 is set. |
| `00959a30` | `PlayerEffectEvent::DeletingDtorAdjustor ? → PlayerEffectEvent::ScalarDeletingDt…` | 🟡 | dtor | Subtracts 0x50 from ECX and jumps to 00957ef0; bytes identify an adjustor thunk. |
| `0095c230` | `PlayerEffectEvent::PlayerEffectEvent` | ✅ | ctor | Calls the base initializer, installs PlayerEffectEvent vtables, initializes event identifiers and flags, copies a global four-word vector, and zeros trailing state fields. |
| `0095c370` | `PlayerEffectEvent::Process` | ✅ | virtual | Checks the current effect handle, optionally stops it, creates an effect through the virtual factory, initializes transform/state, updates the handle, and evaluates completion or cancellation flags. |
