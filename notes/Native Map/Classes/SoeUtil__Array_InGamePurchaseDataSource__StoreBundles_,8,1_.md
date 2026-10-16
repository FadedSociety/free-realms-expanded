# Class: SoeUtil::Array<InGamePurchaseDataSource::StoreBundles*,8,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0099ba30` | `ArrayStoreBundlesPtr8Slot2FreeOutOfLine` | 🟢 | virtual | Thiscall (ECX=this) with one stack arg param_2, virtual slot [2] of SoeUtil::Array<InGamePurchaseDataSource::StoreBundles*,8,1>. If param_2 differs from (this+0x13) rounded up to a multiple of 4, calls 00401c00(param_2,… |
| `0099db30` | `ArrayStoreBundlesInline8Dtor` | ✅ | dtor | Installs the inline-eight array vtable and clears its count; frees the backing pointer only when it differs from inline storage, then nulls it. It switches to the zero-inline base vtable, clears the count again, passes … |
| `0099dbf0` | `ArrayStoreBundlesAllocate` | ✅ | virtual | Returns inline storage aligned at this+0x13 and sets capacity 8 when the requested count is at most 8; otherwise calls the failure helper. |
| `0099f4e0` | `ArrayStoreBundlesCtorInline8` | ✅ | ctor | Installs the zero-inline array vtable, zeroes data/count/capacity, then installs the inline-capacity-eight array vtable. |
| `0099f600` | `ArrayStoreBundlesInline8Dtor` | ✅ | dtor | Calls the zero-inline array destructor and then frees the heap-allocated array object when the deleting flag is set. |
| `0099f790` | `SwapArrayContents` | ✅ | virtual | Copies the second array into a temporary array, clears its count, copies the first array into the second, clears the first count, then copies the temporary contents into the first. The helper 0099DC20 performs each appe… |
