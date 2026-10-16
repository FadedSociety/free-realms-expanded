# Class: SoeUtil::Hash<ContentPack,128,128>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01047c60` | `Hash_ContentPack128_128::Ctor` | ✅ | ctor | Fastcall, ECX = this. Calls 01047960 (the HashMap<int,ContentPack,128,128> constructor body) and then stores SoeUtil::Hash<ContentPack,128,128>::vftable (0187f47c) at *this. Returns this. |
| `01047cc0` | `Hash_ContentPack128_128::ScalarDeletingDtor` | ✅ | dtor | Thiscall with a flag byte param_2. Calls 01046f00 (the 128/128 map destructor body), then if bit 0 of the flag is set calls _free on the object. Returns the object pointer. |
