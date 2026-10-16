# Class: ChainedActionState

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0147a920` | `ChainedActionState::Enter` | ✅ | virtual | Thiscall, no stack args. Pushes the literal Entering ChainedActionState, calls 0145c4d0 with ECX=this (it asserts mMachine at +8 and returns the machine's +4 field), then calls 013508b0 with that value and the literal (… |
| `0147a960` | `ChainedActionState::ForwardToSlot15` | 🟢 | virtual | Loads the vtable of ECX and tail-jumps through its slot 15 (offset 0x3c) with ECX unchanged. Bytes: mov eax,[ecx]; mov edx,[eax+3ch]; jmp edx. |
| `0147acf0` | `ChainedActionState::SerializeState` | 🟢 | virtual | Thiscall with one stack argument, ret 4. Gets the output object from vtable slot 1 when the argument is 0, otherwise from slot 2 with the argument. Writes framing text through 013c26e0 (no-op) and labelled fields throug… |
| `0147aff0` | `ChainedActionState::TypeName` | ✅ | virtual | No arguments. Returns a pointer to the literal ChainedActionState (mov eax,018f4298; ret). |
| `0147b000` | `ChainedActionState::ReadGlobal01cbe5a8` | 🟢 | virtual | No arguments. Returns the 32-bit global at 01cbe5a8 (mov eax,[01cbe5a8]; ret). The global is uninitialised .data in the file. |
| `0147b010` | `ChainedActionState::WriteGlobal01cbe5a8` | 🟢 | virtual | One stack argument, ret 4. Stores the argument to the global at 01cbe5a8. |
