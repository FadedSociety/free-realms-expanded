# Class: CharacterBase

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01044480` | `CharacterBase::FieldAddr_05ec` | 🟢 | virtual | Thiscall, no stack args. Bytes 8d 81 ec 05 00 00 c3 = lea eax,[ecx+0x5ec]; ret. Returns the address this+0x5ec (not the value stored there). |
| `01044490` | `CharacterBase::SetFloat_05ec` | 🟢 | virtual | Thiscall, one stack arg (pointer p). Bytes: mov eax,[esp+4]; fld dword [eax]; fstp dword [ecx+0x5ec]; ret 4. Copies the 4-byte float at *p into this+0x5ec (x87 load/store). |
| `010444a0` | `CharacterBase::EntryTagName` | ✅ | virtual | No args. Bytes b8 60 f1 87 01 c3 = mov eax,0x0187f160; ret. Returns a pointer to the literal string "entryTag". |
| `010444b0` | `CharacterBase::FieldAddr_05f0` | 🟢 | virtual | Thiscall, no stack args. Bytes 8d 81 f0 05 00 00 c3 = lea eax,[ecx+0x5f0]; ret. Returns the address this+0x5f0. |
| `010444c0` | `CharacterBase::SetFloatFromPtr_05f0` | 🟢 | virtual | Thiscall, one stack arg (pointer p). Bytes: mov eax,[esp+4]; fld dword [eax]; fstp dword [ecx+0x5f0]; ret 4. Copies the 4-byte float at *p into this+0x5f0. |
| `010444d0` | `CharacterBase::FieldAddr_05f4` | 🟢 | virtual | Thiscall, no stack args. Bytes 8d 81 f4 05 00 00 c3 = lea eax,[ecx+0x5f4]; ret. Returns the address this+0x5f4. |
| `010444e0` | `CharacterBase::SetFloat_05f4` | 🟢 | virtual | Thiscall, one stack arg (pointer p). Bytes: mov eax,[esp+4]; fld dword [eax]; fstp dword [ecx+0x5f4]; ret 4. Copies the 4-byte float at *p into this+0x5f4. |
| `010444f0` | `CharacterBase::EntryTagName2` | ✅ | virtual | No args. Bytes b8 60 f1 87 01 c3 = mov eax,0x0187f160; ret. Same body as 010444a0: returns a pointer to the literal "entryTag". |
| `01044500` | `CharacterBase::AdjustorThunk_016c` | ✅ | virtual | Thunk. Bytes 81 c1 6c 01 00 00 e9 95 83 fd ff = add ecx,0x16c; jmp 0101c8a0. Adds 0x16c to this and jumps to 0101c8a0; it does no work of its own. |
| `01044510` | `CharacterBase::FieldAddr_016c` | 🟢 | virtual | Thiscall, no stack args. Bytes 8d 81 6c 01 00 00 c3 = lea eax,[ecx+0x16c]; ret. Returns the address this+0x16c. |
| `01044520` | `CharacterBase::SetDwordPair_0010_0014` | 🟢 | virtual | Thiscall, two stack args (ret 8). Bytes: mov eax,[esp+4]; mov edx,[esp+8]; mov [ecx+0x10],eax; mov [ecx+0x14],edx; ret 8. Stores the two dword arguments into this+0x10 and this+0x14. |
| `01044540` | `CharacterBase::GetDword_0114` | 🟢 | virtual | Thiscall, no args. Bytes 8b 81 14 01 00 00 c3 = mov eax,[ecx+0x114]; ret. Returns the dword at this+0x114. |
| `01044550` | `CharacterBase::ReadDword_StackArg_0090 ?` | 🟡 | virtual | One stack arg, ret 4. Bytes 8b 44 24 04 8b 80 90 00 00 00 c2 04 00 = mov eax,[esp+4]; mov eax,[eax+0x90]; ret 4. Returns the dword at arg+0x90 and does not read ECX. |
| `01044560` | `CharacterBase::CreatePetRecord` | ✅ | virtual | Thiscall. Allocates 0x1f8 bytes with operator new (00d833a6, pushed size 0x1f8), and if non-null calls 010428d0 with the block in ECX. Stores param_2 at block+4, clears the byte at block+8, stores param_3 at block+0x120… |
| `01044650` | `— ⚠reject` | ⚪ | virtual | Thiscall with seven stack params (Ghidra). Calls 00703540, then formats param_3 with the format string at 01770978 (bytes 25 64 00 = "%d") through 00402fd0 into a stack string buffer. Then calls the virtual at this vtab… |
| `01044710` | `CharacterBase::FormatCounterPlus1Into34` | 🟢 | virtual | Thiscall with one stack arg. Calls the virtual at this vtable+0xcc (slot 51) with param_2; if it returns non-null object p: calls 0072d5d0 on p+0x38, reads the int at the returned pointer, formats (that int + 1) with "%… |
| `01044760` | `CharacterBase::FormatCounterMinus1Into34` | 🟢 | virtual | Thiscall with one stack arg. Same as 01044710 except the formatted value is (int read through 0072d5d0 on p+0x38) minus 1. Calls the virtual at this vtable+0xcc (slot 51) with param_2; returns 1 if the object is non-nul… |
| `010449e0` | `CharacterBase::DtorBody` | ✅ | dtor | Fastcall, ECX = this. Sets *this = CharacterBase::vftable, then repeatedly calls 008cace0 on the head at this+0x1d0 while it is non-null (ListRemoveNotify). Calls 01041360, then calls 010448f0 with ECX = this+0x1e0, the… |
| `01044b90` | `CharacterBase::ScalarDeletingDtor` | ✅ | dtor | Thiscall with a flag byte param_2 (bytes 56 8b f1 e8 .. f6 44 24 08 01 74 ..): saves this, calls the dtor body 010449e0, then if bit 0 of the flag is set calls _free on the object. Returns the object pointer. |
