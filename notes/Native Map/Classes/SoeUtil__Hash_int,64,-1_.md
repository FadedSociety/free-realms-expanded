# Class: SoeUtil::Hash<int,64,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c71140` | `SoeUtilHash64_Ctor` | ✅ | ctor | Stores the HashMap<int,int,64,-1> base vtable, zeroes word [2], memsets 0x100 bytes at +0xc (64 bucket heads), zeroes word [1], then stores SoeUtil::Hash<int,64,-1>::vftable. Returns this. |
| `00c711c0` | `SoeUtilHash64::ScalarDeletingDtor` | ✅ | dtor | Stores the HashMap<int,int,64,-1> base vtable, calls 00c6f7e0 (64-bucket node release), then calls _free(this) through thunk 00d82463 when bit 0 of the second argument is set. Returns this. |
