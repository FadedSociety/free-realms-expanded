# Class: Button

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c280f0` | `Button::SetText` | 🟢 | method | Lua method body: requires exactly 2 arguments, guards argument 1 with 00c28090, reads the string argument with 00e39c30(L,2,0), converts it through 0083ce80 into a temporary wrapper whose length is set by 004e884e, and … |
| `00c281b0` | `Button::SetProperty` | 🟢 | method | Lua method body: requires exactly 3 arguments, guards argument 1 with 00c28090, reads both string arguments (00e39c30 at indices 2 and 3), resolves the property name with 00820460 and, when the id is not negative, calls… |
| `00c28220` | `— ⚠reject` | ⚪ | method | Lua method body: guards argument 1 with 00c28090 and, when exactly 3 arguments are present, reads the two integers at stack indices -1 and -2 through 00e3a7b0 and calls the UI command helper 004072e2(0,0,second,first,5)… |
| `00c28270` | `Button::SetFocus` | 🟢 | method | Lua method body: guards argument 1 with 00c28090, zeroes a 7-dword structure plus word (seeded from DAT_01bc3b40) and passes it to the button's virtual method at vtable offset 0x114; when the returned first dword has an… |
| `00c28470` | `RegisterButtonModule` | ✅ | init | Lua module initialiser: calls the register helper 00e39e40(L,'Button',table@0x0182f3d8) whose entries are SetFocus->00c28270, Show->00c28320, Hide->00c28350, GetText->00c28760, SetText->00c280f0, GetSize->00c28610, SetS… |
| `00c28610` | `Button::GetSize` | 🟢 | method | Lua method body: guards argument 1 with 00c28090, then pushes the 32-bit fields at control+0x70 and control+0x74 as Lua numbers through 00e3aa30 and returns 2 Lua results; returns 0 when the guard fails. |
| `00c28650` | `Button::GetProperty` | 🟢 | method | Lua method body: requires exactly 2 arguments, guards argument 1 with 00c28090, resolves the property name string (00e39c30 + 00820460) and, when the id is not negative, builds a SoeUtil::IString<unsigned_short> tempora… |
| `00c28760` | `Button::GetText` | 🟢 | method | Lua method body: guards argument 1 with 00c28090, builds a SoeUtil::IString<char> temporary (empty data 01bc7e20) and reads the button text through 006066d6 with the length helper 0083d1e0, pushes the resulting C string… |
