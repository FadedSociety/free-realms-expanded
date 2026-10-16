# Class: ImageControl

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c26bb0` | `ImageControl::getSize` | 🟢 | method | Lua method body: calls the shared ImageControl userdata checker 00c267a0 with stack index 1; when that returns the wrapped control pointer it pushes the two 32-bit fields at control+0x70 and control+0x74 through the Lua… |
| `00c26bf0` | `ImageControl::getPosition` | 🟢 | method | Lua method body: calls the shared ImageControl userdata checker 00c267a0 with stack index 1; when it returns the wrapped control pointer it pushes the two 32-bit fields at control+0x68 and control+0x6c through the Lua n… |
