# Class: GroupInMinigamePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a3ab0` | `GroupInMinigamePacket::~GroupInMinigamePacket` | 🟢 | dtor | Destructor body: restores BaseGroupPacket::vftable then BasePacket::vftable; dispatcher 0093daf0 destroys the packet with it after constructing the class from FUN_008b75d0 (group sub 17). |
| `008b75d0` | `GroupInMinigamePacket::GroupInMinigamePacket` | ✅ | ctor | Calls the group family initializer for top-level opcode 40 008a35c0(0x11), installs GroupInMinigamePacket::vftable, stores param_2 into +0x10, param_3 into +0x14 and the byte param_4 at +0x18. |
| `008b7640` | `GroupInMinigamePacket::ScalarDeletingDestructor` | 🟢 | dtor | Stores BaseGroupPacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
