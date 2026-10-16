# Class: PacketClientFlashTimer

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089c100` | `PacketClientFlashTimerCtor` | ✅ | ctor | Constructor of PacketClientFlashTimer: installs BasePacket::vftable, stores opcode 0xb1 (177) at +4, installs the derived vtable, clears the byte at +8 and writes a full dword zero at +0xc. |
| `008b1910` | `PacketClientFlashTimer::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor in slot 0 of PacketClientFlashTimer::vftable: writes BasePacket::vftable at offset 0 inside an SEH frame and frees this when flag bit 0 is set. |
