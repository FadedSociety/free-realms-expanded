# Class: SoeUtil::Array<SoundDefinition*,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c1b490` | `SoeUtil::Array::GetReplacementBufferWithGrowthPolicy` | 🟢 | method | Reads capacity at this+0xc. In non-exact mode, a request above capacity selects request*5/4; otherwise it reuses the buffer through the request*4/3 threshold or selects request*6/5. Exact mode reuses only equal capacity… |
| `00c1b550` | `SoeUtil::Array::SwapOrReleaseBuffer` | 🟢 | method | When the flag is set, calls a virtual operation on the supplied buffer object; otherwise swaps the data pointer and conditionally swaps the array count and capacity fields. |
| `00c1bab0` | `SoeUtil::Array::Dtor` | ✅ | dtor | Restores the array vptr, clears its count, frees the backing pointer buffer with alignment 4, and nulls the buffer field. |
| `00c1bae0` | `SoeUtil::Array::ScalarDeletingDtor` | ✅ | dtor | Restores the array vptr, clears its count, frees the backing buffer at alignment four, nulls the buffer pointer, and frees the object when the deleting flag is set. |
| `00c1bc10` | `SoeUtil::Array::SetCapacity` | 🟢 | method | Requests a new buffer through the array's virtual allocator, copies the existing elements, releases the old buffer, and updates the data and capacity fields. |
