# Class: SoeUtil::Array<LobbyDataSourceManager::GameConfigOptionDataSource*,10,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a233d0` | `— ⚠reject` | ⚪ | virtual | Frees the buffer pointer argument with size 4 only when it differs from the array's own inline storage region (this+0x13 rounded down to a 4-byte boundary) - the heap-versus-inline guard used when an inline-capacity arr… |
| `00a239d0` | `— ⚠reject` | ⚪ | dtor | Destructor body of the inline-capacity-10 array: stores the 10,1 vtable, frees the buffer only when it is not the inline region (this+0x13 rounded to 4 bytes) and clears it, then chains through the 0,1 vtable and frees … |
| `00a23b00` | `SoeUtil::ArrayInlineReserve` | ✅ | virtual | Storage hook of the inline-capacity-10 array: when the requested capacity exceeds 10 it tail-calls the heap version 00a232a0; otherwise it reports capacity 10 through the out-parameter and returns the address of the arr… |
| `00a23f40` | `GameConfigOptionArrayCtor` | 🟢 | ctor | Constructor of an object holding the config-option pointer array: stores the 0,1 instantiation vtable, zeroes the buffer/size/capacity fields at +4/+8/+0xc, then stores the 10,1 instantiation vtable so the object ends u… |
| `00a23fa0` | `SoeUtil::ArrayInlineDeletingDtor → SoeUtil::Array<LobbyDataSourceManager::GameC…` | ✅ | dtor | Scalar deleting destructor of the inline-capacity array: calls the destructor body 00a239d0 (which releases the heap buffer if it is not the inline region and chains down through the 0,1 instantiation), then calls _free… |
| `00a24500` | `— ⚠reject` | ⚪ | virtual | Calls the range helper to clear the passed and current arrays, zeros both counts, invokes the helper with a zero-length range, then calls the shared cleanup routine before restoring SEH state. |
