# Class: ItemClassDefinitionManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `010483f0` | `ItemClassDefinitionManager::Ctor` | ✅ | ctor | Fastcall, ECX = this. Stores BaseDefinitionManager::vftable, then ItemClassDefinitionManager::vftable (0187f5e4) at *this, zeroes this+8, +0xc, +0x10 and +4, memsets 0x50 bytes at this+0x14, and returns this. |
| `01048500` | `ItemClassDefinitionManager::ReleaseTable` | ✅ | virtual | Bytes 83 c1 04 e9 48 36 89 ff = add ecx,4; jmp 008dbb50. Thunk: advances this by 4 and tail-jumps to 008dbb50 (the 20-bucket table destroy on the table at this+4 in its own frame). |
| `01048510` | `ItemClassDefinitionManager::DtorBody` | ✅ | dtor | Fastcall, ECX = this. Stores ItemClassDefinitionManager::vftable at *this, calls 008dbb50 (destroys the 20-bucket table), then stores BaseDefinitionManager::vftable (017e8c9c) at *this. |
| `01048570` | `ItemClassDefinitionManager::LoadDefinitions` | ✅ | virtual | Thiscall, one stack arg (path or source IString). Releases the old table through 008dbb50, assigns the source string and appends the literal "ItemClasses.txt" through 00402ce0 / 00402770, then reads lines of up to 0x800… |
| `01048790` | `ItemClassDefinitionManager::ScalarDeletingDtor` | ✅ | dtor | Thiscall with a flag byte param_2. Bytes: push esi; mov esi,ecx; call 01048510 (dtor body); test byte [esp+8],1; je; push 0; push esi; call 00401c00 (free with alignment 0); mov eax,esi; pop esi; ret 4. Frees the object… |
