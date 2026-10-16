# Class: SoeUtil::HashMap<int,int,128,128>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d022c0` | `HashMapIntInt128ScalarDtor` | ✅ | dtor | Scalar deleting destructor: saves this, calls the base body 00d01850, frees the object with _free when bit 0 of the flag argument is set (test byte [esp+8],1), returns this with ret 4. |
| `00d022e0` | `HashMapIntInt128Ctor` | ✅ | ctor | Stores the HashMap<int,int,128,-1> vtable, sets +0x8 to 0 and zeroes the 0x200-byte bucket array at +0xc (_memset of 0x200 bytes), sets +0x4 to 0, stores the HashMap<int,int,128,128> vtable, then calls 00d00640 for base… |
