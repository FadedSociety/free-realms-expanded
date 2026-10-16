# Class: SoeUtil::Hash<ItemCategoryGroupDefinition*,20,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `010450b0` | `Hash_ItemCategoryGroupDefinition::Slot1Dispatch ?` | 🟡 | virtual | No args. Calls 006fb9c0, then 006fb860; if that returns non-null p, calls the virtual at vtable+4 of *p and returns. Otherwise, if global DAT_01bdf280 is non-null, calls its vtable+4 and returns. Otherwise calls 006fb95… |
| `010450f0` | `Hash_ItemCategoryGroupDefinition::FreeBlock4` | 🟢 | virtual | One stack arg (ret 4). Bytes 8b 44 24 04 6a 04 50 e8 .. 83 c4 08 c2 04 00 = push 4; push arg; call 00401c00; ret 4. Passes the argument pointer and alignment 4 to 00401c00 (the memory free routine). The vtable slot [2] … |
| `010452d0` | `Hash_ItemCategoryGroupDefinition::Ctor` | ✅ | ctor | Fastcall, ECX = this. Stores the HashMap vtable (0187f1ec) at *this, zeroes this+8 and this+4, memsets 0x50 bytes at this+0xc, then stores the Hash<ItemCategoryGroupDefinition*,20,-1> vtable (0187f1fc) at *this. Returns… |
| `01045350` | `Hash_ItemCategoryGroupDefinition::ScalarDeletingDtor` | ✅ | dtor | Thiscall with a flag byte param_2. Stores the HashMap vtable (0187f1ec, not the Hash vtable) at *this, calls 008abd80 (20-bucket drain), then if bit 0 of the flag is set calls _free on the object. Returns the object poi… |
