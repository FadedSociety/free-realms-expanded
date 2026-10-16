# Class: ClientAreaDataManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0090bd50` | `ClientAreaDataManager::ClientAreaDataManager` | ✅ | ctor | Calls 008d1970, installs the manager vtables, clears fields +0x2c..+0x38 and a 0x200-byte region, stores param_2 at +0x23c, then calls 00906880. |
| `0090be30` | `ClientAreaDataManager::ScalarDeletingDtor ?` | 🟡 | dtor | Calls 00b0c320 to destroy the manager, frees the object when flag bit 0 is set, and returns its pointer. |
| `0090be80` | `ClientAreaDataManager::ScalarDeletingDtorAdjustor` | ✅ | virtual | Subtracts four from this and jumps to 0090be30, adapting the secondary-vtable this pointer to the deleting destructor. |
| `0090be90` | `ClientAreaDataManager::HasEntries` | ✅ | virtual | Returns whether the signed dword at this+0x38 is greater than zero. |
| `0090bea0` | `ClientAreaDataManager::GetField30` | ✅ | virtual | Returns the dword stored at this+0x30. |
| `0090beb0` | `ClientAreaDataManager::GetField164` | ✅ | virtual | Returns the dword stored at this+0x164. |
| `0090bec0` | `ClientAreaDataManager::GetCount` | ✅ | virtual | Returns the dword stored at this+0x38. |
| `0090c190` | `ClientAreaDataManager::AddStringEntry` | ✅ | virtual | Hashes and inserts the supplied string key, updates the maximum numeric index, stores the index in the new entry, then calls virtual slot 6 with the index and param_4. |
| `00b0ac10` | `ClientAreaDataManager::Reset` | ✅ | virtual | Clears manager flags and conditionally walks its area list, unregistering and deleting eligible entries; full-clear mode delegates to list removal. |
| `00b0c140` | `ClientAreaDataManager::CreateShape` | ✅ | virtual | Matches a shape string against sphere, box, dome, or floor, allocates the concrete type, and inserts it into the manager. |
| `00b0c320` | `ClientAreaDataManager::Dtor` | ✅ | dtor | Restores manager vtables, resets a mode byte, clears area entries, destroys the name hash map, and invokes subordinate cleanup routines. |
