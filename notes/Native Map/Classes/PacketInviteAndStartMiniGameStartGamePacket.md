# Class: PacketInviteAndStartMiniGameStartGamePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008b10a0` | `PacketInviteAndStartMiniGameStartGamePacket::PacketInviteAndStartMiniGameStartG…` | ✅ | ctor | Constructor of the start-game subclass: inside an SEH frame it calls the base constructor 008b0fb0 with argument 3 (which sets packet id 0xb4 and the payload fields) and then overwrites offset 0 with PacketInviteAndStar… |
| `008b1100` | `PacketInviteAndStartMiniGameStartGamePacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor in slot 0 of PacketInviteAndStartMiniGameStartGamePacket::vftable: writes BasePacket::vftable at offset 0 inside an SEH frame and frees this when flag bit 0 is set. |
