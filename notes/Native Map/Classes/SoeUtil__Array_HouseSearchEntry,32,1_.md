# Class: SoeUtil::Array<HouseSearchEntry,32,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa9680` | `SoeUtil::Array::FreeStorageIfHeap` | ✅ | virtual | Virtual release hook of the inline-capacity SoeUtil::Array instantiation: frees (pointer, 8) through FUN_00401c00 only when the pointer differs from the object's inline storage at ((this+0x17) & 0xfffffff8). |
| `00ab7dc0` | `SoeUtil_Array_HouseSearchEntry_32_1::Dtor` | ✅ | dtor | Destructor body for the 32-inline HouseSearchEntry array: stores vftable 018192f0, releases +8 via FUN_00ab28e0, frees +4 with FUN_00401c00 alignment 8 unless it is the inline storage ((this+0x17) rounded to 8), then re… |
| `00ab7e90` | `SoeUtil_Array_HouseSearchEntry_32_1::AcquireBuffer` | ✅ | virtual | Virtual slot 1 of the 32-inline array: for requests of 0x20 or fewer elements it writes capacity 0x20 to the out parameter and returns the inline storage (this+0x17) rounded to 8; otherwise it tail-jumps to FUN_00aa9550… |
| `00ab9bb0` | `SoeUtil_Array_HouseSearchEntry_32_1::Ctor` | ✅ | ctor | Stamps the base SoeUtil::Array<HouseSearchEntry,0,1> vftable 01818b04, zeroes the buffer, count and capacity at +4, +8 and +0xc, then stamps the 32-inline vftable 018192f0 as the final vftable. |
| `00ab9c30` | `SoeUtil_Array_HouseSearchEntry_32_1::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0 of the 32-inline variant): calls FUN_00ab7dc0 on this, then calls _free when bit 0 of the flag argument is set, and returns this. |
| `00abc550` | `— ⚠reject` | ⚪ | virtual | Virtual slot 3 of the 32-inline HouseSearchEntry array. Copies the source range into a stack temporary, appends this array range into the source, clears this, then appends the saved source range into this. The final zer… |
