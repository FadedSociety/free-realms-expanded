# Class: SoeUtil::Array<DelayedAssetLoader::PendingRequest*,4,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ae3f50` | `— ⚠reject` | ⚪ | virtual | When requested count exceeds four delegates to dynamic array allocator; otherwise returns aligned inline storage at object+0x13 and reports capacity four. |
| `00ae4540` | `SmallPointerArrayDtor` | ✅ | dtor | Installs small-array vtable, frees dynamic backing when not inline storage, clears fields, then runs dynamic array cleanup and frees its buffer if needed. |
| `00ae4600` | `— ⚠reject` | ⚪ | virtual | For capacity at most four, stores capacity four and returns aligned inline storage at object+0x13; larger requests delegate to dynamic allocation hook. |
| `00ae4670` | `SmallPointerArray::ScalarDeletingDtor` | ✅ | dtor | Calls small-array destructor, optionally frees this on flag bit 0, and returns this otherwise. |
| `00ae4700` | `SmallPointerArrayInit` | ✅ | ctor | Initializes dynamic array fields to zero, then installs the four-inline-slot array vtable. |
| `00ae5390` | `— ⚠reject` | ⚪ | virtual | Appends the current range from the second array to its own range, resets that array length, appends its previous range to the destination array, resets its own length, and issues an empty append. |
