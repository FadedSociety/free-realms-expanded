# Class: SoundGroup

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c1bb40` | `SoundGroup::Ctor` | ✅ | ctor | Constructs a SoundDefinitionAssetLoader base, initializes multiple SoundGroup vptrs plus the embedded array vptr, and zeroes four array fields. |
| `00c1bc80` | `SoundGroup::ScalarDeletingDtor` | ✅ | dtor | Adjusts the this pointer by -0x10 and jumps to the shared SoundGroup deleting-destructor body at 00c1db60. |
| `00c1bc90` | `SoundGroup::ScalarDeletingDtor` | ✅ | dtor | Adjusts the this pointer by -8 and jumps to the shared SoundGroup deleting-destructor body at 00c1db60. |
| `00c1bd70` | `SoundGroup::FindEntryPointerIndex` | 🟢 | method | Scans the pointer array at this+0x1c over count at this+0x20, compares each entry pointer with the argument and returns the first matching index or -1. |
| `00c1d210` | `SoundGroup::ClearEntries` | 🟢 | method | Walks the counted pointer array, invokes the entry release virtual for non-null elements, and sets the count field at +0x20 to zero. |
| `00c1d2f0` | `SoundGroup::Remove ⚠downgrade` | 🟢 | method | Finds matching IDs in the array, invokes each matching object's virtual release method, shifts later entries left, and decrements the count. |
| `00c1da10` | `SoundGroup::Dtor → SoundGroup::DtorBody` | ✅ | dtor | Restores SoundGroup vptrs, clears entries, restores the embedded array vptr, frees the array storage, restores RefCounted vptr and calls its cleanup routine. |
| `00c1dac0` | `SoundGroup::SetEntryAtIndex → SoundGroup::InsertEntryAtIndex` | 🟢 | method | For a nonnegative index, increments the entry reference count, fills intervening array positions with null, stores the entry at that index and extends the high-water count when needed. |
| `00c1db30` | `SoundGroup::SetEntryAtIndex` | 🟢 | method | If the supplied index is negative, replaces it with count at this+0x20; calls 007cd920 with the entry and chosen index, then calls 00c1dac0 with the same values to store it. The registration work inside 007cd920 is cond… |
| `00c1db60` | `SoundGroup::ScalarDeletingDtor` | ✅ | dtor | Calls the SoundGroup destructor body, frees the object when the deleting flag is set, and returns the original pointer otherwise. |
