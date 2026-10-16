# Class: ControllerSoccer

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00afb7a0` | `ControllerSoccer::ControllerSoccer` | ✅ | ctor | Thiscall ctor, three stack args forwarded to base 0094f6c0, returns this (ECX=this). Stores ControllerSoccer::vftable at +0x00, initialises about 0x7e dwords of state (float constants from DAT_0175b420, DAT_017ea718, DA… |
| `00afbc60` | `— ⚠reject` | ⚪ | virtual | Thiscall with one int arg (time step), vtable slot 51 (+0xcc). Returns unless the object at this+0x74 -> +0x120 -> +0x21c is nonzero. Reads 01071fc0 and sets flag +0x1c4 when it changes. Computes a clamped interpolation… |
