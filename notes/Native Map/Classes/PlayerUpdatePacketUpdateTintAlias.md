# Class: PlayerUpdatePacketUpdateTintAlias

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008d04d0` | `PlayerUpdatePacketUpdateTintAlias::PlayerUpdatePacketUpdateTintAlias` | ✅ | ctor | Constructor of the UpdateTintAlias player-update packet: runs the family constructor FUN_0089fbb0 with subtype 0x34 (52), installs PlayerUpdatePacketUpdateTintAlias::vftable and initializes one embedded SoeUtil::IString… |
| `008dcb20` | `PlayerUpdatePacketUpdateTintAlias::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: runs the destructor body 008d0540 (player-update packet destructor with an IString member at +0x18) and frees the object with _free (00d82463) when the low bit of the flag argument is set. |
