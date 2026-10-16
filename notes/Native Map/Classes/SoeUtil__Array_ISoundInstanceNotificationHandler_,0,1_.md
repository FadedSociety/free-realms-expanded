# Class: SoeUtil::Array<ISoundInstanceNotificationHandler*,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `007be3d0` | `ArrayPlanCapacityAndAlloc` | ✅ | virtual | Virtual slot [1] (thiscall, three stack args, ret 0xc): reads the current capacity at this+0xc; with flag 0 chooses a new count of req*5/4 when capacity < req, keeps the current buffer at this+4 when capacity <= req*4/3… |
| `007be470` | `ArrayFreeBuffer` | ✅ | virtual | Virtual slot [2] (ret 8, two stack args): frees the buffer pointer in the first argument through 00401c00(ptr, 4) (alignment 4) and cleans up the two argument slots; the second argument is unused. |
| `007be490` | `ArraySwapState` | ✅ | virtual | Virtual slot [3] (thiscall, two stack args, ret 8). With the flag byte (second arg) non-zero it tail-jumps to the other object's vtable slot +0xc with ECX = other, args (this, 0). With flag zero it exchanges the three s… |
| `007bf200` | `ArrayDtorBody` | ✅ | dtor | Fastcall (ECX = array): sets the vtable to SoeUtil::Array<ISoundInstanceNotificationHandler*,0,1>, zeroes the count at +8, frees the buffer at +4 through 00401c00(ptr, 4) and zeroes +4. Frees nothing else and does not f… |
| `007bf320` | `ArrayScalarDeletingDtor` | ✅ | dtor | Virtual slot [0] (thiscall, one byte flag arg, ret 4): sets the vtable, zeroes the count, frees the buffer at +4 through 00401c00(ptr, 4) and zeroes +4; when the flag bit 0 is set it calls _free on the object; returns t… |
