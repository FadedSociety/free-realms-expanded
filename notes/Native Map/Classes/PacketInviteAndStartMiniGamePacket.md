# Class: PacketInviteAndStartMiniGamePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008b0fb0` | `PacketInviteAndStartMiniGamePacket::PacketInviteAndStartMiniGamePacket` | ✅ | ctor | Packet constructor: writes BasePacket::vftable, stores the packet id 0xb4 (180) at this+4, overwrites offset 0 with PacketInviteAndStartMiniGamePacket::vftable, saves the argument at this+8, copies the two global dwords… |
| `008b1040` | `PacketInviteAndStartMiniGamePacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor in slot 0 of PacketInviteAndStartMiniGamePacket::vftable: writes BasePacket::vftable at offset 0 inside an SEH frame and frees this when flag bit 0 is set. |
