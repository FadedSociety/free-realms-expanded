# Class: ProjectileManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b83d20` | `ProjectileManager::Ctor` | ✅ | ctor | Thiscall constructor with three stack arguments after this. Stores ProjectileManagerInterface::vftable and then ProjectileManager::vftable into this, zeroes the dword at +0x08, +0x0c and +0x10, memsets 400 bytes at +0x1… |
| `00b84050` | `ProjectileManager::ClearAll` | 🟢 | method | Fastcall with this in ECX. Walks the whole list from this+0x08 without any flag test. For each node it unlinks it from its bucket (this+0x14 + (key % 100) * 4), unlinks it from the list (prev at +0x18, next at +0x1c, he… |
| `00b84100` | `ProjectileManager::~ProjectileManager` | ✅ | dtor | Non-deleting destructor body. Stores ProjectileManager::vftable into this, calls ClearAll (00b84050) on the list, then calls the base destructor body FUN_010c17e0 (the ProjectileManagerInterface body, per 00b83bb0). It … |
| `00b84590` | `ProjectileManager::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor. Calls the dtor body FUN_00b84100 on ECX (this), then calls _free on this when bit 0 of the flag argument is set, and returns this. |
| `00b84e90` | `— ⚠reject` | ⚪ | virtual | Per-update steering pass of ProjectileManager (vtable slot 1): for the owner object param_2 it resolves this+0x1ac->+200 into a world/actor pointer, then either (a) with a target list at param_3+8 non-empty, averages th… |
