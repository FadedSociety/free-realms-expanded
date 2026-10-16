# Class: SoeUtil::Array<AppPhysics::ConvexHit,64,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa9e90` | `SoeUtil::Array::FreeStorageIfHeap` | ✅ | virtual | Virtual release hook of the inline-capacity SoeUtil::Array instantiation: frees (pointer, 0x10) through FUN_00401c00 only when the pointer differs from the object's inline storage at ((this+0x1f) & 0xfffffff0). |
| `00aabdb0` | `SoeUtil::Array::~Array` | ✅ | dtor | Two-stage destructor of the inline-capacity array: resets this+0 to the 64-slot vtable, clears the size, frees the buffer only when it differs from the inline storage at ((this+0x1f) & 0xfffffff0), clears the buffer wor… |
| `00aabf00` | `SoeUtil::Array::Reserve` | ✅ | virtual | Inline-capacity reservation slot: when more than 0x40 elements are requested it tail-calls the dynamic reservation FUN_00aa9d60, otherwise it reports the inline capacity 0x40 through param_3 and returns the inline stora… |
| `00aaf360` | `ConvexHitArrayCtor` | ✅ | ctor | Initializes the inline array header, zeros size, capacity, and storage, then installs the 64-element ConvexHit array vtable. |
| `00aaf400` | `ConvexHitArrayScalarDeletingDtor` | ✅ | dtor | Calls the array destructor, frees the object only when flag bit zero is set, and otherwise returns the original object pointer. |
| `00ab26a0` | `ConvexHitArraySlot3` | ✅ | virtual | Runs the slot-three array operation over source and destination 0x30-byte records, clears both size fields, and invokes the allocator cleanup path. |
