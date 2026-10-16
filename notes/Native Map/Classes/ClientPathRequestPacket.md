# Class: ClientPathRequestPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008b7df0` | `ClientPathRequestPacket::ClientPathRequestPacket` | ✅ | ctor | Calls ClientPathBasePacket constructor with sub-opcode 1, installs ClientPathRequestPacket::vftable, copies two globals, and zeroes its remaining payload fields except two initialized defaults. |
| `008b7ea0` | `ClientPathRequestPacket::ScalarDeletingDtor` | ✅ | dtor | Restores BasePacket::vftable and frees the object when the deleting flag bit is set. |
