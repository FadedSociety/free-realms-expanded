# Class: PlayerSwitchProfileEvent

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bfb4d0` | `PlayerSwitchProfileEvent::PlayerSwitchProfileEvent` | ✅ | ctor | Calls the base-event initializer, installs PlayerSwitchProfileEvent::vftable, clears the byte at object offset 0x50, and returns this. |
| `00bfb550` | `PlayerSwitchProfileEvent::~PlayerSwitchProfileEvent` | ✅ | dtor | Stores the PlayerSwitchProfileEvent vtable, changes it to PlayerEvent, then calls 007e7980 with ECX=this for base cleanup; it does not free the object. |
| `00bfb5c0` | `PlayerSwitchProfileEvent::GetFlag50` | ✅ | virtual | Returns the byte stored at this+0x50. |
| `00bfb5e0` | `PlayerSwitchProfileEvent::SetFlag50WhenBit0` | ✅ | virtual | If param_3 bit 0 is set, writes 1 to this+0x50; always returns 0. |
| `00bfb600` | `PlayerSwitchProfileEvent::DeleteWhenFlagSet` | ✅ | dtor | Calls the event destructor body, frees the object only when param_2 bit 0 is set, and returns the original pointer otherwise. |
