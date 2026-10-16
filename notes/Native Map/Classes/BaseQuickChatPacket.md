# Class: BaseQuickChatPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b8acb0` | `— ⚠reject` | ⚪ | ctor | Installs BasePacket then BaseQuickChatPacket vtables, stores opcode 0x43, and initializes the subtype field to zero. |
| `00b8ad10` | `BaseQuickChatPacket::CtorWithSubtype` | ✅ | ctor | Installs BasePacket and BaseQuickChatPacket vtables, stores opcode 0x43, and copies the supplied subtype into field 2. |
| `00b8ad80` | `BaseQuickChatPacket::Dtor` | ✅ | dtor | Reinstalls the BaseQuickChatPacket vtable and then the BasePacket vtable at object offset zero. |
| `00b8add0` | `BaseQuickChatPacket::ScalarDeletingDtor` | ✅ | dtor | Restores quick-chat and base packet vtables, then frees the object when flag bit zero is set. |
| `00b8aeb0` | `BaseQuickChatPacket::DtorBase` | ✅ | dtor | Reinstalls BaseQuickChatPacket and BasePacket vtables at the object's first word. |
| `00b93ad0` | `BaseQuickChatPacket::ReadBaseFields` | ✅ | handler | Reads two 16-bit header values, then reads one dword and two more dwords into offsets 0x0c, 0x10, and 0x14; exhausted reads zero fields and set the cursor error flag. |
| `00b966e0` | `BaseQuickChatPacket_Unpack` | ✅ | handler | Handler for a received quick-chat packet: overlays BaseQuickChatPacket on the stack object (00b8acb0), reads the two header words from the wire buffer (00b90640), then branches on the subtype word stored at +8: subtype … |
