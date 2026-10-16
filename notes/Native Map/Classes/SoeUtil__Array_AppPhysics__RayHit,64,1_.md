# Class: SoeUtil::Array<AppPhysics::RayHit,64,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa9cf0` | `SoeUtil::Array::FreeStorageIfHeap` | ✅ | virtual | Virtual release hook of the inline-capacity SoeUtil::Array instantiation: frees (pointer, 0x10) through FUN_00401c00 only when the pointer differs from the object's inline storage at ((this+0x1f) & 0xfffffff0). |
| `00aabbe0` | `SoeUtil::Array::~Array` | ✅ | dtor | Two-stage destructor of the inline-capacity array: resets this+0 to the 64-slot vtable, clears the size, frees the buffer only when it differs from the inline storage at ((this+0x1f) & 0xfffffff0), clears the buffer wor… |
| `00aabd30` | `SoeUtil::Array::Reserve` | ✅ | virtual | Inline-capacity reservation slot: when more than 0x40 elements are requested it tail-calls the dynamic reservation FUN_00a2a310, otherwise it reports the inline capacity 0x40 through param_3 and returns the inline stora… |
| `00aaf300` | `RayHitArrayCtor` | ✅ | ctor | Initializes the inline array header, zeros size, capacity, and storage, then installs the 64-element RayHit array vtable. |
| `00aaf3e0` | `RayHitArrayScalarDeletingDtor` | ✅ | dtor | Calls the array destructor, frees the object only when flag bit zero is set, and otherwise returns the original object pointer. |
| `00ab25b0` | `RayHitArraySlot3` | ✅ | virtual | Runs the slot-three array operation over source and destination 0x30-byte records, clears both size fields, and invokes the allocator cleanup path. |
