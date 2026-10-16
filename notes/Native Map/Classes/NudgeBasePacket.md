# Class: NudgeBasePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a41310` | `NudgeBasePacket::NudgeBasePacket` | ✅ | ctor | Base constructor of the nudge packet family: installs BasePacket::vftable, stores the family opcode 0xcb at +4, installs NudgeBasePacket::vftable and stores the caller's sub-opcode at +8. |
| `00a41380` | `NudgeBasePacket::~NudgeBasePacket` | 🟢 | dtor | Destructor body that only reinstalls BasePacket::vftable at +0, i.e. the base-part teardown of a nudge packet object built on the stack. |
| `00a41440` | `NudgeBasePacket::~NudgeBasePacket` | 🟢 | dtor | Second copy of the base-part teardown: reinstalls BasePacket::vftable at +0 and returns. |
| `00a416c0` | `NudgeBasePacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0) of NudgeBasePacket: installs BasePacket::vftable and frees the object with _free when bit 0 of the delete flag is set. |
