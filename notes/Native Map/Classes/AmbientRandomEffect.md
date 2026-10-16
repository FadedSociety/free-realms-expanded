# Class: AmbientRandomEffect

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009f6c10` | `AmbientRandomEffect::DtorBody` | ✅ | dtor | Destructor body without free: fastcall, writes AmbientRandomEffect::vftable (0x0181110c) into [this] and returns. |
| `009f6c20` | `AmbientRandomEffect::ReadFloat50 ?` | 🟡 | virtual | Virtual slot [1] of AmbientRandomEffect::vftable: returns the float at this+0x50 on the x87 stack (fld dword [ecx+0x50]; ret, 4 bytes). |
| `009f6c90` | `AmbientRandomEffect::RaiseFloat50 ? ⚠rename` | 🟡 | virtual | Virtual slot [2] of AmbientRandomEffect::vftable, thiscall with one float argument (ret 4): when the argument is greater than the float constant at DAT_01762a30 (comiss against it), it stores the argument at this+0x50; … |
| `009f6cb0` | `AmbientRandomEffect::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor, slot [0] of AmbientRandomEffect::vftable: stores AmbientRandomEffect::vftable into [this], then calls _free(this) through thunk 00d82463 only when bit 0 of param_2 is set. Returns this. |
| `009f6f80` | `AmbientRandomEffect::InitVtableOnly ?` | 🟡 | init | Fastcall, SEH-framed: stores AmbientRandomEffect::vftable into [this] and restores the SEH chain. No other field is written and no callee is made. |
| `009f74a0` | `AmbientRandomEffect::CopyCtor` | ✅ | ctor | Thiscall with one source pointer (ret 4). Installs AmbientRandomEffect::vftable into [this], then copies source fields field by field with no callee: dwords +0x10..+0x20 to this+0x10..+0x20, bytes +0x24..+0x27 to this+0… |
| `009f75e0` | `AmbientRandomEffect::ScalarDeletingDtor_Alt` | ✅ | dtor | Scalar deleting destructor body of the same shape as slot [0] of AmbientRandomEffect::vftable: stores AmbientRandomEffect::vftable into [this], sets the EH state to -1, and calls _free(this) through thunk 00d82463 only … |
| `009f7640` | `AmbientRandomEffect::DefaultCtor` | ✅ | ctor | Fastcall default constructor: installs AmbientRandomEffect::vftable, zeroes most fields, loads global constants into +0x14, +0x18, +0x1c, +0x20, +0x3c, +0x50 and +0x6c (DAT_017eb904, DAT_01811154, DAT_017ed858, _DAT_017… |
