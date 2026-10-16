# Class: SoeUtil::Array<HouseKeywordEntry,32,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa9530` | `SoeUtil::Array::FreeStorageIfHeap` | ✅ | virtual | Virtual release hook of the inline-capacity SoeUtil::Array instantiation: frees (pointer, 4) through FUN_00401c00 only when the pointer differs from the object's inline storage at (this+0x10). |
| `00aaba70` | `SoeUtil::Array::~Array` | ✅ | dtor | Two-stage destructor of the inline-capacity array: resets this+0 to the 32-slot vtable, clears the size, frees the buffer only when it differs from the inline storage at (this+0x10), clears the buffer word, then repeats… |
| `00aabb30` | `SoeUtil::Array::Reserve` | ✅ | virtual | Inline-capacity reservation slot: when more than 0x20 elements are requested it tail-calls the dynamic reservation FUN_00aa9400, otherwise it reports the inline capacity 0x20 through param_3 and returns the inline stora… |
| `00aaf2a0` | `HouseKeywordArrayCtor` | ✅ | ctor | Initializes the inline array header and zeroes its size, capacity, and storage fields, then installs the 32-element specialization vtable. |
| `00aaf3c0` | `HouseKeywordArrayScalarDeletingDtor` | ✅ | dtor | Calls the array destructor, frees the object only when flag bit zero is set, and otherwise returns the original object pointer. |
| `00aaf530` | `— ⚠reject` | ⚪ | virtual | Runs the slot-three array routine over source and destination contents, resets their size fields, and invokes the specialization's cleanup helper. |
