# Class: SoeUtil::Array<SoeUtil::Math::Vector<float,2>,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a53420` | `— ⚠reject` | ⚪ | virtual | Virtual allocate hook (slot 1) of Array<SoeUtil::Math::Vector<float,2>,0,1>: reads the current capacity at +0xc; when the exact flag is clear and the capacity is below the request the new capacity becomes request*5/4, w… |
| `00a534c0` | `SoeUtil::Array_Vector2f_Deallocate` | ✅ | virtual | Virtual deallocate hook (slot 2) of Array<SoeUtil::Math::Vector<float,2>,0,1>: the 18-byte body loads the pointer argument (8b 44 24 04), pushes alignment 4 (6a 04) and forwards to FUN_00401c00, cleaning two stack argum… |
| `00a534e0` | `SoeUtil::Array_Vector2f_Swap ⚠downgrade` | 🟢 | virtual | Virtual swap hook (slot 3) of Array<SoeUtil::Math::Vector<float,2>,0,1>: with a zero flag it exchanges the buffer pointer at +4 and, when the values differ, XOR-exchanges the count at +8 and capacity at +0xc with the ar… |
| `00a559b0` | `SoeUtil::Array_Vector2f_Dtor` | ✅ | dtor | Installs the float-vector array vtable, clears its count, frees the backing storage with alignment four, and resets the pointer. |
| `00a56430` | `SoeUtil::Array_Vector2f_ScalarDeletingDtor` | ✅ | dtor | Resets the Vector2 array vtable, clears its count, frees the backing buffer with alignment four, and conditionally frees the object. |
| `00cc7130` | `Vector2ArrayDtor` | ✅ | dtor | Installs the Vector2 array vtable, resets count, frees storage with alignment four, and clears the storage pointer. |
| `00cc7570` | `Vector2ArrayCtor ⚠downgrade` | 🟢 | ctor | Installs the Vector2 array vtable and zeros its data pointer, count, capacity, and inline-storage fields. |
