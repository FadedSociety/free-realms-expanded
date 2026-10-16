# Class: SoeUtil::Array<int,512,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0081b990` | `ArrayInt512_Ctor` | ✅ | ctor | Constructor (no args): stores SoeUtil::Array<int,0,1>::vftable at [0], zeroes dwords [1], [2], [3] (count/capacity/data), then stores SoeUtil::Array<int,512,1>::vftable at [0]. |
| `0081b9f0` | `ArrayInt512_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (thiscall, one stack byte flag): calls the destructor body 0081b000 on this; if flag bit 0 is set it frees this through the _free thunk; returns this. |
| `0081c730` | `— ⚠reject` | ⚪ | virtual | Thiscall (ecx=this) with one stack arg. Builds a temporary Array<int,0,1> from the argument through 0073c310, exchanges contents with this and the argument through 0073c370 calls, then frees the temporary buffer through… |
