# Class: PlayerMotionEvent

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008b82a0` | `PlayerMotionEvent::PlayerMotionEvent` | ✅ | ctor | Calls PlayerEvent base construction, installs PlayerMotionEvent::vftable, and stores the supplied dword at offset +0x50. |
| `008b8330` | `PlayerMotionEvent::DtorBody` | ✅ | dtor | Restores PlayerMotionEvent then PlayerEvent vtables, calls FUN_007e7980 with param_1 in ECX, restores ExceptionList, and returns. |
| `008b8390` | `— ⚠reject` | ⚪ | virtual | Examines param_3 flag bits, conditionally calls the event and client callbacks using the dword at +0x50, and returns a masked status byte. |
| `008b8400` | `PlayerMotionEvent::ScalarDeletingDtor` | ✅ | dtor | Calls PlayerMotionEvent cleanup and frees the object when the deleting flag bit is set. |
