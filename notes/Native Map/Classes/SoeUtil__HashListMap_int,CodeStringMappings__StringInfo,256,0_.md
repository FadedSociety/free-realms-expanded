# Class: SoeUtil::HashListMap<int,CodeStringMappings::StringInfo,256,0>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009209e0` | `HashListMap_CodeStringInfo_Dtor` | ✅ | dtor | Destructor body: stores the HashListMap<int,CodeStringMappings::StringInfo,256,0> vtable, drains the node at this+8 through 008facc0 until empty, frees the two words at +8/+0xc of the pointer stored at this+0x414 throug… |
| `00920e10` | `HashListMap_CodeStringInfo_ScalarDtor` | ✅ | dtor | Scalar deleting destructor: calls the body 009209e0 on this, then frees the object through _free when bit 0 of the second argument is set; returns this. |
| `00921230` | `HashListMap_CodeStringInfo_Ctor` | ✅ | ctor | Constructor: stores the HashListMap<...,256,-1> vtable at this, zeroes +4, +8, +0xc and +0x10, memsets the 0x400-byte bucket array at +0x14, then stores the HashListMap<int,CodeStringMappings::StringInfo,256,0> vtable a… |
