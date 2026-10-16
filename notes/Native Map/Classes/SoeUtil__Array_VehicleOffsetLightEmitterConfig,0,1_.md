# Class: SoeUtil::Array<VehicleOffsetLightEmitterConfig,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009d3060` | `VehicleOffsetLightEmitterConfigArray::Allocate` | ✅ | virtual | With mode byte zero, capacity below request selects about 5/4 of the request; otherwise capacity at or below floor(4*request/3) is reused, and larger capacity selects floor(6*request/5), which can shrink the buffer. Wit… |
| `009d3100` | `VehicleOffsetLightEmitterConfigArray::Free` | ✅ | virtual | Frees the supplied array pointer through FUN_00401c00 with 16-byte alignment. |
| `009d3120` | `VehicleOffsetLightEmitterConfigArray::SwapOrDispatch` | 🟢 | virtual | When the mode byte is zero, exchanges the data pointer, capacity, and count with the supplied array. For nonzero mode, tail-dispatches through the source vtable slot +0xc with the source as ECX, this as an argument, and… |
| `009d37b0` | `VehicleOffsetLightEmitterConfigArray::Dtor` | ✅ | dtor | Restores the typed array vtable, clears its count, frees its buffer with 16-byte alignment, and nulls the buffer pointer. |
| `009d38d0` | `VehicleOffsetLightEmitterConfigArray::ScalarDeletingDtor` | ✅ | dtor | Restores the typed array vtable, clears count, frees its storage, and frees the object when the deleting flag is set. |
