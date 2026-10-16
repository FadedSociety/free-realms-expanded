# Class: SoeUtil::Array<InGamePurchaseDataSource::StoreBundles*,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0099b900` | `— ⚠reject` | ⚪ | virtual | Virtual slot [1] of SoeUtil::Array<InGamePurchaseDataSource::StoreBundles*,0,1>, thiscall with three stack args (count, out-pointer, flag byte). Same capacity policy as 0099b0f0: flag zero grows to count*6/5 or shrinks … |
| `0099b9a0` | `ArrayStoreBundlesPtrSlot2FreeBuffer` | 🟢 | virtual | Virtual slot [2] of SoeUtil::Array<InGamePurchaseDataSource::StoreBundles*,0,1>, one stack arg (buffer), 18 bytes. Calls 00401c00(buffer,4) and returns void. |
| `0099b9c0` | `ArrayStoreBundlesPtrSlot3SwapOrRelease` | 🟢 | virtual | Virtual slot [3] of SoeUtil::Array<InGamePurchaseDataSource::StoreBundles*,0,1>, thiscall with two stack args (other array param_2, byte flag). If the flag is nonzero, calls vtable slot +0xc of *param_2 and returns. Oth… |
| `0099db00` | `ArrayStoreBundlesDtor` | ✅ | dtor | Restores the array vtable, clears size, frees its backing storage with element alignment 4, and nulls the allocation pointer. |
| `0099e660` | `ArrayStoreBundlesScalarDeletingDtor` | ✅ | virtual | Restores the StoreBundles array vtable, releases backing storage, clears pointer and count, and optionally frees the array object. |
