# Class: TableControl

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c26d10` | `TableControl::getSelectedRow` | 🟢 | method | Lua method body: guards the argument with 00c26cb0(L,1); when the guard returns a control it calls 00663fa0 to copy the control's selection pair (fields +0x138 and +0x13c) into an 8-byte local buffer and pushes the seco… |
| `00c26e10` | `RegisterTableControlModule` | ✅ | init | Lua module initialiser: calls the register helper 00e39e40(L,'TableControl',table@0x0182f0f0) whose entries are getSelectedRow->00c26d10, getData->00c26fb0, find->00c26ef0; creates the type metatable with the newmetatab… |
| `00c26fb0` | `TableControl::getData` | 🟢 | method | Lua method body: guards with 00c26cb0(L,1); when lua_gettop-style 00e3a390(L) reports exactly 3 arguments it reads two integers with 00e3a7b0(L,-2) and 00e3a7b0(L,-1), constructs a SoeUtil::IString<char> temporary (vfta… |
