# Class: SoeUtil::Array<GFxWrap::GuiGfxParam,1,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009d12a0` | `GFxWrap::GuiGfxParamArray::NodeFree` | ✅ | virtual | Frees the supplied pointer with eight-byte alignment unless it equals the array's inline-buffer address rounded up to eight-byte alignment. |
| `009d1c10` | `GFxWrap::GuiGfxParamArray::Dtor` | ✅ | dtor | Restores inline-array then base-array vtables, destroys stored parameters, frees non-inline storage, and clears its pointer. |
| `009d1cb0` | `GFxWrap::GuiGfxParamArray::Allocate` | ✅ | virtual | For requests of at most one element, returns aligned inline storage and reports capacity one; larger requests delegate to FUN_0083eb60. |
| `009d1ff0` | `GuiGfxParamArrayCtor` | ✅ | ctor | Initializes the base array pointer, count, and capacity, then installs the one-inline-element array vtable. |
| `009d2050` | `GuiGfxParamArray::ScalarDeletingDtor` | ✅ | dtor | Runs the inline-array destructor and frees the object when the deleting flag is set. |
| `009d2070` | `GuiGfxParamArray::SwapContents` | ✅ | virtual | Exchanges the two arrays' contents through a temporary: copies the source into temporary storage, clears the source, copies the old destination into the source, clears the destination, then copies the temporary contents… |
