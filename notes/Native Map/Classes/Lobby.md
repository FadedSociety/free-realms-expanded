# Class: Lobby

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01049900` | `Lobby::Ctor` | ✅ | ctor | Fastcall, ECX = this. Stores Lobby::vftable (0187f65c) at *this, zeroes this+8, +0xc, +0x10 and +4, memsets 400 bytes at this+0x14 to zero, and returns nothing (no return is shown in the pseudocode). |
| `010499a0` | `Lobby::CtorWithArg` | ✅ | ctor | Thiscall, one stack arg (param_2). Stores Lobby::vftable (0187f65c) at *this, zeroes this+8, +0xc, +0x10 and +4, memsets 400 bytes at this+0x14, and stores param_2 at this+0x1a4. Returns this. |
| `01049a20` | `Lobby::EhCleanupSetVtable` | ✅ | dtor | Fastcall, ECX = this. EH-frame prologue and epilogue around a single store: mov [ecx], Lobby::vftable (0187f65c). Nothing else is done; it returns. |
| `01049ab0` | `Lobby::FindThenSlot3` | ✅ | virtual | Thiscall, one stack arg (uint key). Bytes 56 8b 74 24 08 .. mul by 0x51eb851f (key % 100 computed as key - (key/100)*100) .. 8b 44 81 14 = bucket head at this+0x14 + (key%100)*4; walks the chain (next at node+8) compari… |
| `01049af0` | `Lobby::ScalarDeletingDtor` | ✅ | dtor | Thiscall with a flag byte param_2. Stores Lobby::vftable (0187f65c) at *this, then if bit 0 of the flag is set calls _free on the object. Returns the object pointer. No separate dtor body in this function. |
| `01049ba0` | `Lobby::ReleaseNode` | ✅ | virtual | Thiscall, one stack arg (node pointer). Bytes 56 8b 74 24 08 85 f6 74 19 56 83 c1 04 e8 .. = if the node is non-null: calls 008c0220 with ECX = this+4 and the node as its stack arg (accepted label unknown: unlinks it fr… |
