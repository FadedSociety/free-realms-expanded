# Class: MiniGameDataLockedPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009afa50` | `MiniGameDataLockedPacket::Ctor` | ✅ | ctor | Calls minigame base constructor with values 1 and -1 sentinels, installs packet vtable, and initializes three integer arrays. |
| `009afae0` | `MiniGameDataLockedPacket::Dtor` | 🟢 | dtor | Restores array vtables, frees three integer-array buffers, clears their fields, and restores BasePacket vtable. |
| `009b0800` | `MiniGameDataLockedPacket_ScalarDeletingDtor` | ✅ | dtor | Calls packet destructor and conditionally frees the packet object when deletion flag bit zero is set. |
