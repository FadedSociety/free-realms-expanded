# Class: InGamePurchaseDataSource::StoreBundleRoot

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00997790` | `StoreBundleRootCtor` | 🟢 | ctor | Thiscall (ECX=this) with one stack arg param_2, returns this (ret 4). Calls 00822990 with param_2 pushed, then stores the InGamePurchaseDataSource::StoreBundleRoot vtable (0180d308) at [this]. |
| `009977f0` | `StoreBundleRootDtorBody` | 🟢 | dtor | Fastcall (ECX=this) with an SEH frame. Stores the InGamePurchaseDataSource::StoreBundleRoot vtable (0180d308) at [this], then calls 00822810 once (the base teardown; the Ghidra argument is the cookie expression, so the … |
| `0099aa00` | `StoreBundleRootScalarDeletingDtor` | ✅ | dtor | Thiscall (ECX=this) with one stack arg (byte flags) and an SEH frame. Stores the InGamePurchaseDataSource::StoreBundleRoot vtable (0180d308) at [this], calls 00822810 (base teardown), and, if bit 0 of the flags arg is s… |
