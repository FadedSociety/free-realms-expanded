# Class: SoeUtil::HashMap<int,int,1024,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00987970` | `HashMap1024Int_Constructor` | ✅ | ctor | Stores the map vtable, zeros the count and state fields, clears 0x1000 bytes of bucket storage, and returns this. |
| `009881b0` | `HashMap1024Int_ClearThunk` | ✅ | dtor | Writes the map vtable and tail-jumps to 009879a0, which clears the 1024 buckets. |
| `009881c0` | `HashMap1024Int_AllocateNode` | ✅ | virtual | Uses the active/global allocator sequence and requests 0xc bytes aligned to 4 as the fallback allocation. |
| `00988200` | `HashMap1024Int_FreeNode` | ✅ | virtual | Forwards its pointer argument with alignment 4 to 00401c00, then returns. |
| `009884c0` | `HashMap1024Int_ScalarDeletingDtor` | ✅ | dtor | Installs the map vtable, clears entries through 009879a0, optionally frees this object, and otherwise returns it. |
| `00ae43e0` | `HashMapBaseDtor` | ✅ | dtor | Installs the HashMap vtable and invokes hash-table cleanup helper 009879a0. |
| `00ae4430` | `HashMapInit` | ✅ | ctor | Installs map vtable, clears count and 1024 bucket heads, then installs Hash<int,1024,-1> base vtable. |
