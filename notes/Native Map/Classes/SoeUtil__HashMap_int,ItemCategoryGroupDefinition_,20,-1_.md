# Class: SoeUtil::HashMap<int,ItemCategoryGroupDefinition*,20,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01044f90` | `HashMap_ItemCategoryGroupDefinition::Ctor` | ✅ | ctor | Fastcall, ECX = this. Stores SoeUtil::HashMap<int,ItemCategoryGroupDefinition*,20,-1>::vftable (0187f1ec) at *this, zeroes this+8 and this+4, memsets 0x50 bytes at this+0xc to zero, and returns this (EAX). |
| `010450a0` | `HashMap_ItemCategoryGroupDefinition::DtorBody` | ✅ | dtor | Fastcall, ECX = this. Bytes c7 01 .. e9 .. = mov [ecx], 0187f1ec (HashMap vftable), then jmp 008abd80. It is a tail-jump: it stores the vtable and transfers to 008abd80 (the 20-bucket hash drain) without its own return.… |
| `01045110` | `HashMap_ItemCategoryGroupDefinition::ScalarDeletingDtor` | ✅ | dtor | Thiscall with a flag byte param_2. Stores the HashMap vtable (0187f1ec) at *this, calls 008abd80 (20-bucket drain), then if bit 0 of the flag is set calls _free on the object. Returns the object pointer. |
| `01045280` | `HashMap_ItemCategoryGroupDefinition::DtorCleanup` | ✅ | dtor | Fastcall, ECX = this. Stores the HashMap vtable (0187f1ec) at *this, then calls 008abd80 (20-bucket drain) with this as the receiver (the cookie value Ghidra shows is not a real argument). Unlike 010450a0 it runs inside… |
