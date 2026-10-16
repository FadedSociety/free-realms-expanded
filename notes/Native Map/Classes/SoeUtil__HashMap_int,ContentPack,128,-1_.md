# Class: SoeUtil::HashMap<int,ContentPack,128,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `010462a0` | `HashMap_ContentPack::Ctor` | ✅ | ctor | Fastcall, ECX = this. Stores SoeUtil::HashMap<int,ContentPack,128,-1>::vftable (0187f2a4) at *this, zeroes this+8, memsets 0x200 bytes at this+0xc (128 bucket heads), zeroes this+4, and returns this. |
| `01046450` | `HashMap_ContentPack::Slot1Dispatch ?` | 🟡 | virtual | No args. Calls 006fb9c0, then 006fb860; if that returns non-null p, calls the virtual at vtable+4 of *p and returns. Otherwise, if global DAT_01bdf280 is non-null, calls its vtable+4 and returns. Otherwise calls 006fb95… |
| `01046490` | `HashMap_ContentPack::FreeBlock4` | 🟢 | virtual | One stack arg (ret 4). Bytes 8b 44 24 04 6a 04 50 e8 .. 83 c4 08 c2 04 00 = push 4; push arg; call 00401c00; ret 4. Frees the argument with alignment 4 through 00401c00. |
| `01046d20` | `HashMap_ContentPack::DtorBody` | ✅ | dtor | Fastcall, ECX = this. Bytes c7 01 .. e9 .. = mov [ecx], 0187f2a4 (HashMap<int,ContentPack,128,-1> vtable), then jmp 01046b70 (tail-jump to the table clear). No own return. |
| `01046dd0` | `HashMap_ContentPack::ScalarDeletingDtor` | ✅ | dtor | Thiscall with a flag byte. Bytes: push esi; mov esi,ecx; mov [esi], 0187f2a4; call 01046b70 (clear, ECX = esi); test byte [esp+8],1; je; push esi; call _free; mov eax,esi; pop esi. Returns the object pointer (the ret fo… |
