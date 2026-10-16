# Class: CommandPacketInteractionStartWheel

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009aad80` | `CommandPacketInteractionStartWheel::CommandPacketInteractionStartWheel` | ✅ | ctor | Constructor: calls the command base constructor FUN_008a2820(0xb,0) (base opcode 26, sub-opcode 11) and installs CommandPacketInteractionStartWheel::vftable. |
| `009aceb0` | `DeletingDtor_009aceb0 → CommandPacketInteractionStartWheel::ScalarDeletingDestr…` | 🟢 | dtor | Scalar deleting destructor for the BaseCommandPacket-derived interaction-start-wheel packet: rewrites *this to BaseCommandPacket::vftable, then to BasePacket::vftable, and when bit 0 of param_2 is set frees the object t… |
