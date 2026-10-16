# Class: SoeUtil::Array<AppPhysics::ConvexHit,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa9d60` | `SoeUtil::Array::Reserve` | ✅ | virtual | Virtual buffer-acquisition slot of the SoeUtil::Array instantiation: with param_4 false a request above the capacity at this+0xc is raised by 5/4 and a request below 3/4 of capacity is shrunk toward 6/5; with param_4 tr… |
| `00aa9e00` | `SoeUtil::Array::FreeStorage` | ✅ | virtual | Virtual release hook of the SoeUtil::Array instantiation: forwards (pointer, 0x10) to the shared free entry FUN_00401c00 (FreeMemory). |
| `00aa9e20` | `SoeUtil::Array::SwapTripleAndNotify` | ✅ | virtual | Virtual slot [3] of the SoeUtil::Array instantiation: exchanges the three dwords at this+4/+8/+0xc with param_2[1..3] using XOR swaps; when param_3 is non-zero it instead tail-dispatches to param_2's vtable slot +0xc wi… |
| `00aabd80` | `SoeUtil::Array::~Array` | ✅ | dtor | Destructor of the array object: resets this+0 to the instantiation's vtable, clears the size at +8, frees the heap buffer at +4 with 16-byte alignment through FUN_00401c00 and clears the buffer word. |
| `00aac040` | `SoeUtil::Array::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot [0]): resets the vtable, clears the size, frees the buffer at +4 with 16-byte alignment, clears the buffer and frees the object itself when the low bit of param_2 is set. |
