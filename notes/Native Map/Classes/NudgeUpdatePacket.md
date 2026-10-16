# Class: NudgeUpdatePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a413d0` | `NudgeUpdatePacket::NudgeUpdatePacket` | ✅ | ctor | Constructor of the nudge update message: runs the family base constructor FUN_00a41310(4) (opcode 0xcb, sub-opcode 4 at +8) and installs NudgeUpdatePacket::vftable with the four payload dwords at +0xc, +0x10, +0x14 and … |
| `00a41720` | `NudgeUpdatePacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0) of NudgeUpdatePacket: installs BasePacket::vftable and frees the packet with _free when bit 0 of the delete flag is set. |
| `00a421f0` | `SendNudgeUpdateKind1` | 🟢 | method | Builds a NudgeUpdatePacket on the stack with the constructor 00a413d0, fills the payload dwords at +0xc, +0x10 and +0x14 with the three caller arguments and sets +0x18 to the constant 1, then calls 00a420c0(packet, 1) w… |
| `00a42290` | `SendNudgeUpdateKind2` | 🟢 | method | Same nudge update builder as 00a421f0 with the payload constant at +0x18 set to 2: constructs NudgeUpdatePacket (00a413d0), copies the three caller arguments to +0xc/+0x10/+0x14, sets +0x18 = 2 and sends through 00a420c… |
| `00a42330` | `SendNudgeUpdateKind4` | 🟢 | method | Nudge update builder with the payload constant at +0x18 set to 4: constructs the packet (00a413d0), fills +0xc/+0x10/+0x14 from the three arguments, sets +0x18 = 4 and sends through 00a420c0. |
| `00a423d0` | `SendNudgeUpdateKind3` | 🟢 | method | Nudge update builder with the payload constant at +0x18 set to 3: constructs the packet (00a413d0), copies the three caller arguments into +0xc/+0x10/+0x14, sets +0x18 = 3 and sends through 00a420c0. |
