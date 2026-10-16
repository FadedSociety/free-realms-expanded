# Class: SoeUtil::Array<GFxWrap::GuiGfxParam,20,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c254a0` | `GuiGfxParamArrayFreeBuffer` | ✅ | virtual | Frees the supplied buffer with alignment eight unless it equals the object's aligned inline-storage address. |
| `00c25840` | `GuiGfxParamArrayDtor` | ✅ | dtor | Destroys array elements, releases heap storage when not using inline storage, switches to the zero-capacity base vtable, and releases its storage. |
| `00c258e0` | `GuiGfxParamArrayAllocate` | ✅ | virtual | Returns aligned inline storage and capacity 20 for requests of at most 20; for larger requests, jumps to 0083eb60. |
| `00c25910` | `GuiGfxParamArrayCtor` | ✅ | ctor | Initializes the base array vtable, zeros its storage pointer, count, and capacity fields, then installs the 20-element specialization vtable. |
| `00c25970` | `GuiGfxParamArrayScalarDeletingDtor` | ✅ | dtor | Calls the array destructor, frees the object when flag bit zero is set, and otherwise returns the original pointer. |
| `00c25990` | `— ⚠reject` | ⚪ | virtual | Copies the source element sequence through 00844660, destroys destination elements, clears its storage, and resets the temporary array state. |
