# Class: SoeUtil::HashMap<int,SoundOneShotDefinitionLoader::LoadPendingOneShotEntry,32,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008be010` | `SoundOneShotPendingEntryHashMap::Allocate` | ✅ | virtual | Allocates a 0x1c-byte pending entry aligned to 4 after trying the thread and global allocators. |
| `008be050` | `SoundOneShotPendingEntryHashMap::FreeAllocation` | ✅ | virtual | Calls the engine free routine with the supplied pointer and 4-byte alignment. |
| `0090c310` | `HashMapSoundPendingOneShot::Cleanup` | ✅ | dtor | Restores the pending-one-shot hash-map vtable and calls 008fa260 to release its stored map contents. |
| `0090f900` | `SoundOneShotPendingMap::SoundOneShotPendingMap` | ✅ | ctor | Installs the pending-one-shot hash-map vtable, clears the map size and bucket count, zeros its 32 bucket heads, and installs the matching hash vtable. |
| `00910530` | `SoundOneShotPendingMap::ScalarDeletingDtor` | ✅ | dtor | Restores the pending-entry map vtable, calls 008fa260 to clean up map contents, optionally frees the object, and returns it. |
