# Class: PlayerHousingInstanceInfoArray

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cbe720` | `PlayerHousingInstanceInfoArray::DestroyStorage` | ✅ | dtor | Restores the array vtable, releases all live PlayerHousingInstanceInfo entries, frees the backing allocation with alignment 8, and clears its pointer. |
| `00cbe750` | `PlayerHousingInstanceInfoArray::PushCopy` | 🟢 | function | Increments the array size, reserves backing capacity if required, then copy-constructs the supplied record into the new final slot and returns the copy result. |
| `00cbea70` | `PlayerHousingInstanceInfoArray::ScalarDeletingDtor` | ✅ | dtor | Restores the array vtable, releases all live elements, frees backing storage with alignment 8, clears the storage pointer, and frees the object when the low flag bit is set. |
