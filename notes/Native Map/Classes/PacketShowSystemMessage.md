# Class: PacketShowSystemMessage

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089b8e0` | `PacketShowSystemMessageCtor` | ✅ | ctor | Constructor of PacketShowSystemMessage: installs BasePacket::vftable, stores opcode 0x67 (103) at +4, installs the derived vtable and clears +8/+0xc while setting the message style word at +0x10 to 3. |
| `008b14f0` | `PacketShowSystemMessage::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor in slot 0 of PacketShowSystemMessage::vftable: writes BasePacket::vftable at offset 0 inside an SEH frame and frees this when flag bit 0 is set. |
