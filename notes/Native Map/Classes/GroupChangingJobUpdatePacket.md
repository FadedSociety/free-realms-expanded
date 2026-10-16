# Class: GroupChangingJobUpdatePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a3a10` | `GroupChangingJobUpdatePacket::~GroupChangingJobUpdatePacket` | 🟢 | dtor | Destructor body: restores BaseGroupPacket::vftable then BasePacket::vftable; dispatcher 0093daf0 destroys the changing-job update with it after constructing the class from FUN_008b7400 (group sub 15). |
| `008b7400` | `GroupChangingJobUpdatePacket::GroupChangingJobUpdatePacket` | ✅ | ctor | Calls the group family initializer for top-level opcode 40 008a35c0(0xf), installs GroupChangingJobUpdatePacket::vftable, stores param_2 into +0x10, param_3 into +0x14 and the byte param_4 at +0x18. |
| `008b7470` | `GroupChangingJobUpdatePacket::ScalarDeletingDestructor` | 🟢 | dtor | Stores BaseGroupPacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
