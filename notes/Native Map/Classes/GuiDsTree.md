# Class: GuiDsTree

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009974a0` | `GuiDsTree::Ctor` | 🟢 | ctor | Thiscall (ECX=this) with one stack arg param_2, returns this (ret 4). Calls 00822990 with param_2 pushed, stores the GuiDsTree vtable (0180d2b8) at [this], zeroes the dword at +0x30, and stores 1 in the byte at +0x34. |
| `0099a630` | `GuiDsTreeScalarDeletingDtor` | ✅ | dtor | Thiscall (ECX=this) with one stack arg (byte flags). Calls 00822810 (base teardown, ECX=this per the pseudocode; cookie argument artifact) and then, if bit 0 of the flags arg is set, pushes this and calls the free thunk… |
