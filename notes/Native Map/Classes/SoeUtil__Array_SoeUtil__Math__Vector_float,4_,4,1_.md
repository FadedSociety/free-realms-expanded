# Class: SoeUtil::Array<SoeUtil::Math::Vector<float,4>,4,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b003a0` | `ArrayVec4FreeBuffer` | ✅ | virtual | Thiscall with a buffer arg and one unused arg (ret 8, bytes checked). If the buffer differs from the 16-byte-aligned inline storage at (this+0x1f) and 0xfffffff0, calls 00401c00(buffer, 0x10) to free it. |
| `00b009b0` | `— ⚠reject` | ⚪ | dtor | Destructor body of the four-inline-slot vector array. Stores Array<Vector4,4,1>::vftable at +0; when the data pointer at +4 is not the inline buffer ((this+0x1f)&0xfffffff0) it calls FUN_00401c00(data,0x10,cookie); it t… |
| `00b00a40` | `SoeUtil::Array::Reserve` | ✅ | virtual | Virtual slot 1 of the four-element vector array. Raw bytes 8b442404 83f804 7e09 89442404 e9<rel> show that the requested element count is loaded and, when it is greater than 4, control tail-jumps (jmp, no return) to FUN… |
| `00b01180` | `SoeUtil::Array::Array` | ✅ | ctor | Constructor: stores the base Array<Vector4,0,1>::vftable at +0, zeroes +4/+8/+0xc (data pointer, size, capacity) and finally installs Array<Vector4,4,1>::vftable. No allocation takes place. |
| `00b011e0` | `SoeUtil::Array::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor at vtable slot 0. Bytes 56 8bf1 e8<rel> f644240801 7409 56 e8<rel> 83c404 8bc6 5e c20400: calls the destructor body FUN_00b009b0 with the same this, and when the low bit of the flag argum… |
| `00b02790` | `SoeUtil::Array::Swap` | ✅ | virtual | Element-wise exchange of two four-slot vector arrays. Raw bytes show a stack temporary with the base Array<Vector4,0,1>::vftable that receives every element of the other array (param_2: data +4, size +8) through FUN_008… |
