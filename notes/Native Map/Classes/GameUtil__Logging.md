# Class: GameUtil::Logging

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008cf6e0` | `GameUtil::Logging::GameUtil::Logging` | ✅ | ctor | Constructor of the GameUtil::Logging object: installs GameUtil::Logging::vftable, clears the flag byte at +4, initializes three embedded SoeUtil::IString members (+8, +0x18, +0x28) with the shared empty buffer and zero … |
| `008cf7a0` | `GameUtil::Logging::~GameUtil::Logging` | ✅ | dtor | Destructor of GameUtil::Logging: rewrites GameUtil::Logging::vftable and releases the three embedded SoeUtil::IString members at +0x28, +0x18 and +8 (each capacity-checked with a locked refcount decrement and virtual sl… |
| `008cf880` | `GameUtil::Logging::ScalarDeletingDestructor` | 🟢 | dtor | MSVC scalar deleting destructor: runs the GameUtil::Logging destructor body FUN_008cf7a0 and frees the object with _free when bit 0 of the flag argument is set. |
| `008dc940` | `GameUtil::Logging::AppendField18IfPresent` | ✅ | virtual | Virtual method (shared vtable slot 3 of GameUtil::Logging, RewardBundleBase, ClientRewardManager and ClientRewardBundle): when the dword at this+0x20 is set it appends a fixed text constant through SoeUtil::IStringAppen… |
| `008dc970` | `GameUtil::Logging::AppendField28IfPresent` | ✅ | virtual | Virtual method (shared vtable slot 4 of GameUtil::Logging, RewardBundleBase, ClientRewardManager and ClientRewardBundle): when the dword at this+0x30 is set it appends a fixed text constant through SoeUtil::IStringAppen… |
