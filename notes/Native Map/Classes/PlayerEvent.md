# Class: PlayerEvent

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a5510` | `PlayerEvent::~PlayerEvent` | ✅ | dtor | Destructor body restores PlayerEvent::vftable, then calls base event teardown FUN_007e7980 with this in ECX inside an SEH frame; twenty EH funclets call it. |
| `008a5570` | `PlayerEvent::ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor restores PlayerEvent::vftable, calls FUN_007e7980 with this in ECX, then calls _free(this) when param_2 bit 0 is set. |
| `008b8160` | `PlayerEvent::PlayerEvent` | ✅ | ctor | Calls the base event constructor, installs PlayerEvent::vftable, and stores param_3 and param_4 at fields +0x48 and +0x4c. |
