# Class: SoeGems::RefArrayPooled<unsigned_char>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0107fd20` | `RefArrayPooledUcharCtor` | 🟢 | ctor | Constructor body: calls 0083fe90 on this, installs the SoeGems::RefArrayPooled<unsigned_char> vftable at this+0x0 and at this+0x10 (second base), and clears the dword at +0x18. |
| `0107fd80` | `RefArrayReplaceBuffer ?` | 🟡 | method | Thiscall: obtains a new buffer through vslot 1 with (size at +0x8, address of the argument, argument); if it differs from the buffer at +0x4 it copies the size at +0x8 bytes with _memcpy, releases the old buffer through… |
| `01080060` | `RefArrayPooledSecondaryDtorThunk` | 🟢 | virtual | Virtual slot 0 of the secondary vtable: this-adjusting thunk (bytes: sub ecx,0x10; jmp 01080970). Adjusts the receiver by -0x10 and jumps to 01080970, which is not decoded in this shard. |
| `01080590` | `RefArrayPooledDtorBody` | 🟢 | dtor | Non-deleting destructor body: stores the SoeGems::RefArrayPooled<unsigned_char> vftable at this+0x0 and at this+0x10, clears the size dword at +0x8, calls 0107fd80 with argument 1 so the buffer is replaced by a zero-siz… |
| `01080970` | `RefArrayPooledScalarDeletingDtor` | 🟢 | dtor | Virtual slot 0 scalar deleting destructor of the primary vftable: calls the destructor body 01080590 on this, then frees this with _free when flag bit 0 of the argument is set, and returns this. |
| `01081a70` | `— ⚠reject` | ⚪ | virtual | Virtual slot 1 (args: requested size, out capacity pointer, grow flag). Chooses a capacity (1.25x or 1.2x growth, or keeps the current buffer when already sized), stores the size at the out pointer, and when the object … |
| `01081b60` | `RefArrayPooledFreeBuffer` | 🟢 | virtual | Virtual slot 2 (args: buffer, size, flag): when the flag bit 0 is set frees the buffer with 00401c00 (FreeMemory alignment 1); otherwise initialises the singleton when needed and returns the buffer to the pool through 0… |
