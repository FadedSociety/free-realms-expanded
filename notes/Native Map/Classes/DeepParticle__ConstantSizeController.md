# Class: DeepParticle::ConstantSizeController

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00884a60` | `ConstantSizeController::FillRange` | ✅ | virtual | Thiscall virtual, slot [5]. Stack args: owner object (param_2), begin (param_3), end (param_4); ret 0xc. Reads one dword from the owner table at owner.+0xc.+0x148.+0x24 indexed by the short at this+0x12, then, when begi… |
| `00885510` | `ConstantSizeController::ctor` | ✅ | ctor | Thiscall constructor, three stack args; ret 0xc. Calls FUN_008850a0 on this (SizeInitializer constructor), stores DeepParticle::ConstantSizeController::vftable, sets the 16-bit fields at this+0x10 and +0x12 to 0xffff, r… |
| `008858a0` | `ConstantSizeController::ctorAppend` | ✅ | ctor | Thiscall constructor, args: value (param_2) and owner object (param_3); ret 8. Calls FUN_008850a0 on this (SizeInitializer constructor), stores DeepParticle::ConstantSizeController::vftable, sets the 16-bit fields at th… |
