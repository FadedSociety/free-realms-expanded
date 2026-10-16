# Class: SoeUtil::Array<MessageDataSource::MessageData,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cfa640` | `MessageDataArrayReserve ?` | 🟡 | virtual | Chooses a new capacity based on current capacity, requested size, and a flag; returns existing storage when sufficient or allocates count times 0x14 bytes. |
| `00cfa6e0` | `MessageDataArrayFree ?` | 🟡 | virtual | Calls the shared memory free routine with alignment 4 for the supplied pointer. |
| `00cfa700` | `MessageDataArraySwapOrDestroy ?` | 🟡 | virtual | If the flag is set, calls a virtual destruction method; otherwise swaps the data pointer, size, and capacity fields with the supplied array. |
| `00cfaa20` | `MessageDataArrayCleanup ?` | 🟡 | dtor | Restores the array vtable, destroys all live elements, frees the backing buffer with alignment 4, and clears the pointer. |
| `00cfaa50` | `MessageDataArrayScalarDeletingDtor ?` | 🟡 | dtor | Restores the array vtable, destroys all elements, frees the backing buffer, and conditionally frees the array object. |
