# Class: ItemCategoryManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01045520` | `ItemCategoryManager::Ctor` | ✅ | ctor | Thiscall, one stack arg (param_2, stored at this+0x260). Stores BaseDefinitionManager::vftable, then ItemCategoryManager::vftable at *this. Zeroes this+8, +0xc, +0x10, memsets 400 bytes at this+0x14, zeroes this+4, +0x1… |
| `010455f0` | `ItemCategoryManager::ReleaseTables` | ✅ | virtual | Thiscall with no args. Bytes: push esi; mov esi,ecx; lea ecx,[esi+4]; call 008dbc70; lea ecx,[esi+0x204]; call 008abd80; lea ecx,[esi+0x1a4]; pop esi; jmp 008dbda0. Releases the table at this+4, drains the Hash sub-obje… |
| `01045620` | `ItemCategoryManager::DtorBody` | ✅ | dtor | Fastcall, ECX = this. Stores ItemCategoryManager::vftable at *this, calls 008dbc70 (table at this+4), 008abd80 and 008dbda0 (table at this+0x1a4), stores SoeUtil::HashMap<int,ItemCategoryGroupDefinition*,20,-1>::vftable… |
| `010456c0` | `— ⚠reject` | ⚪ | virtual | Thiscall, one stack arg (path string, copied into a string at the start). Calls 008dbc70 first (a 100-bucket table destroy; the receiver is not verified in the bytes). Reads ItemCategories.txt through 00fbf4a0 with sepa… |
| `01045ba0` | `ItemCategoryManager::ScalarDeletingDtor` | ✅ | dtor | Thiscall with a flag byte param_2. Bytes: push esi; mov esi,ecx; call 01045620 (dtor body); test byte [esp+8],1; je; push 0; push esi; call 00401c00 (free with alignment 0); add esp,8; mov eax,esi; pop esi; ret 4. Frees… |
