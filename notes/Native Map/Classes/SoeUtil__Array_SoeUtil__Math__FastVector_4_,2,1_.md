# Class: SoeUtil::Array<SoeUtil::Math::FastVector<4>,2,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a88760` | `ArrayFreeHeapBuffer` | ✅ | virtual | Virtual slot 2 (+8) of SoeUtil::Array<FastVector<4>,2,1>: if the buffer pointer param_2 differs from (this+0x1f) rounded down to 16, calls 00401c00 to free it with alignment 0x10; returns with ret 8. |
| `00a89440` | `ArrayInlineOrGrowSlot1` | ✅ | virtual | Virtual slot 1 (+4) of SoeUtil::Array<FastVector<4>,2,1>: if the requested count (param_2) is above 2 it tail-jumps to 007f8800; otherwise it stores 2 into *param_3 and returns the 16-byte-aligned inline buffer address … |
| `00a8a910` | `Array_FastVector4_2_1::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor of SoeUtil::Array<FastVector<4>,2,1>: calls 00a893b0 (the array destructor body), then calls _free(this) through thunk 00d82463 when bit 0 of param_2 is set; returns this. |
| `00a8a970` | `ArrayFastVector4VSlot3 ?` | 🟡 | virtual | Virtual slot 3 (+0xc) of SoeUtil::Array<FastVector<4>,2,1>: passes the data and count of param_2 to 007f8ce0 and zeroes param_2+8, passes the data and count of this to 007f8ce0 and zeroes this+8, calls 007f8ce0(0,0), th… |
