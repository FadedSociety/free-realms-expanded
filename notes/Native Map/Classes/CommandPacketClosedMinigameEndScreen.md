# Class: CommandPacketClosedMinigameEndScreen

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a2eb0` | `CommandPacketClosedMinigameEndScreen::CommandPacketClosedMinigameEndScreen` | ✅ | ctor | Constructor: calls FUN_008a2820(0x2a,0) so the packet is command opcode 26 with sub-opcode 42, then installs CommandPacketClosedMinigameEndScreen::vftable inside an SEH frame. |
| `008b6d30` | `CommandPacketClosedMinigameEndScreen::ScalarDeletingDestructor` | 🟢 | dtor | Stores BaseCommandPacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
