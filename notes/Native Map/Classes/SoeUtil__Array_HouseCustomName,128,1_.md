# Class: SoeUtil::Array<HouseCustomName,128,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa93d0` | `SoeUtil::Array::FreeStorageIfHeap` | ✅ | virtual | Virtual release hook of the inline-capacity SoeUtil::Array instantiation: frees (pointer, 4) through FUN_00401c00 only when the pointer differs from the object's inline storage at (this+0x10). |
| `00ab7be0` | `SoeUtil_Array_HouseCustomName_128_1::Dtor` | ✅ | dtor | Destructor body for the 128-inline HouseCustomName array: stores vftable 018192dc, calls FUN_00ab28a0 on +8, frees the +4 buffer through FUN_00401c00 unless it equals the inline storage (this+0x13 rounded up to 4), then… |
| `00ab7cb0` | `SoeUtil_Array_HouseCustomName_128_1::AcquireBuffer` | ✅ | virtual | Virtual slot 1 of the 128-inline array: when the requested count is 0x80 or less it writes capacity 0x80 to the out parameter and returns the inline storage at (this+0x13) rounded up to 4; otherwise it tail-jumps to FUN… |
| `00ab9b50` | `SoeUtil_Array_HouseCustomName_128_1::Ctor` | ✅ | ctor | Stamps the base SoeUtil::Array<HouseCustomName,0,1> vftable 01818adc, zeroes the buffer, count and capacity at +4, +8 and +0xc, then stamps the 128-inline vftable 018192dc as the final vftable. |
| `00ab9c10` | `SoeUtil_Array_HouseCustomName_128_1::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0 of the 128-inline variant): calls FUN_00ab7be0 on this, then calls _free when bit 0 of the flag argument is set, and returns this. |
| `00ab9cb0` | `SoeUtil_Array_HouseCustomName_128_1::Swap` | 🟢 | virtual | Virtual slot 3 of the 128-inline HouseCustomName array. Copies the source range into a stack temporary, appends this array range into the source, clears this, then appends the saved source range into this. The final zer… |
