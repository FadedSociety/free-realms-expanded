# Class: GTreeCtrl

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0067b32f` | `GTreeCtrl::ParentWalkThenVirtual11c ?` | 🟡 | method | Thiscall with (char param_2, int param_3) and ret 0xc. When param_2 is 0 it walks p = param_3, p = 00406665(p) while p is nonzero, returning early if p equals this. Then, if [this+0x1c4] is nonzero, it calls that object… |
| `0069a47e` | `—` | ⚪ | unknown | Virtual slot [159] shared by GuiTreePrivateCtrl and GTreeCtrl (both vtable slots hold 0069a47e). Body (18012 bytes of pseudocode) was not read in this pass. |
| `006a5ebb` | `— ⚠reject` | ⚪ | method | Thiscall with one stack argument, ret 4. Pushes the argument, then [this+0xf4], then [this+0xf0], calls 006a2b97 with no stack cleanup of its own, then returns with ret 4. EAX is not set. |
