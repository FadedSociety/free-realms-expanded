# Class: SoeUtil::Array<SoeUtil::Math::FastVector<4>,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00809b30` | `ArrayFastVector4Ctor` | ✅ | ctor | Fastcall ctor (ECX = this): stores SoeUtil::Array<SoeUtil::Math::FastVector<4>,0,1>::vftable at this+0 and zeroes the dwords at this+4 through this+0x14 (five dwords); no call. |
| `00809b90` | `ArrayFastVector4DtorBody` | 🟢 | dtor | Fastcall SEH-framed teardown: stores the FastVector<4> array vtable at this+0, clears this+8, frees the buffer at this+4 with 00401c00(buffer, 16) (alignment 16) and clears this+4. |
| `00a893b0` | `— ⚠reject` | ⚪ | dtor | Destructor body for SoeUtil::Array<FastVector<4>,2,1>: sets the vptr to the ,2,1 vtable, frees the element buffer at param_1[1] through 00401c00 with alignment 0x10 unless it is the aligned inline buffer (this+0x1f roun… |
| `00a8a8b0` | `Array_FastVector4_0_1::Ctor` | ✅ | ctor | Constructs a SoeUtil::Array<FastVector<4>,2,1> object: stores the base SoeUtil::Array<FastVector<4>,0,1> vtable at *param_1, clears param_1[1], param_1[2] and param_1[3], then stores the ,2,1 vtable at *param_1 as the f… |
