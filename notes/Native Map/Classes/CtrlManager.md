# Class: CtrlManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0043a470` | `CtrlManager::CtrlManager` | ✅ | ctor | Runs base and record initialization, installs the CtrlManager vtable set, clears its manager fields and lists, sets +0xec to 0x40, releases the referenced fields, allocates and initializes the +0xf4 table with 0x800 ent… |
| `0043a78f` | `CtrlManager::CtrlManagerVariant` | 🟢 | ctor | Runs the base and record initialization, installs the same vtable set as the primary constructor, initializes the lists and manager fields, releases the referenced fields, leaves +0xf4 null, then calls the alternate reg… |
| `0043d7ea` | `CtrlManager::~CtrlManager` | 🟢 | dtor | Rewrites the CtrlManager vtable fields, tears down the manager entry list and +0xf4 buffer, destroys the 0x30-stride vector and tree containers, releases interface fields at +0x150,+0x154,+0x158,+0x15c,+0x160,+0x164,+0x… |
| `0043f8e0` | `— ⚠reject` | ⚪ | function | Mode-selector factory for the 0x1F0-byte control-manager class: mode 0 constructs in place (FUN_0043A470); mode 1 copy-constructs in place from param_3 (FUN_0043A78F); mode 2 allocates 0x1F0 (FUN_0040823C) then construc… |
| `0043f971` | `CtrlManager::ScalarDeletingDtor` | 🟢 | dtor | Calls CtrlManager destructor body 0043D7EA; when param_2 bit 0 is set, calls the shared free helper through thunk 00408259; otherwise returns param_1. The function uses RET 4. |
