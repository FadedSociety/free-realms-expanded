# Class: PlayerUpdatePacketCustomizeNpc

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008d0330` | `PlayerUpdatePacketCustomizeNpc::PlayerUpdatePacketCustomizeNpc` | ✅ | ctor | Constructor of the CustomizeNpc player-update packet: runs the family constructor FUN_0089fbb0 with the stack immediate 0x2e (46) as the sub-opcode, installs PlayerUpdatePacketCustomizeNpc::vftable and then the three-st… |
| `008dcae0` | `PlayerUpdatePacketCustomizeNpc::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: runs the destructor body 008d03a0 (a player-update packet destructor) and frees the object with _free (00d82463) when the low bit of the flag argument is set. |
