# Class: LabelControl

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c271b0` | `LabelControl::setText` | 🟢 | method | Lua method body: requires exactly 2 arguments (lua_gettop-style 00e3a390), guards argument 1 with 00c27150, reads the string argument with 00e39c30(L,2,0), wraps the resulting C string through 0083ce80 into a temporary … |
| `00c27320` | `RegisterLabelControlModule` | ✅ | init | Lua module initialiser: calls the register helper 00e39e40(L,'LabelControl',table@0x0182f198) whose entries are getText->00c274c0, setText->00c271b0 and find->00c27400; creates the type metatable with 00e39010(L,'LabelC… |
| `00c274c0` | `LabelControl::getText` | 🟢 | method | Lua method body: requires exactly 1 argument, guards argument 1 with 00c27150, builds a SoeUtil::IString<char> temporary (vftable 0175b24c, empty-string data 01bc7e20) and asks the control for its text through 00609d6c … |
