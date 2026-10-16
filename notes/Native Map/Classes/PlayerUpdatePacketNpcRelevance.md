# Class: PlayerUpdatePacketNpcRelevance

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008d08b0` | `PlayerUpdatePacketNpcRelevance::PlayerUpdatePacketNpcRelevance` | ✅ | ctor | Constructor of the NpcRelevance player-update packet: runs the family constructor FUN_0089fbb0 with subtype 0xc (12), installs PlayerUpdatePacketNpcRelevance::vftable, then installs the SoeUtil::Array<PlayerUpdatePacket… |
| `008d0920` | `PlayerUpdatePacketNpcRelevance::~PlayerUpdatePacketNpcRelevance` | ✅ | dtor | Destructor body of the NpcRelevance player-update packet: rewrites the embedded SoeUtil::Array<PlayerUpdatePacketNpcRelevance::Entry> vtable at +0xc (param_1[3]), frees the array buffer at +0x10 (param_1[4]) through the… |
| `008dcbf0` | `PlayerUpdatePacketNpcRelevance::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: runs the destructor body 008d0920 and frees the object with _free (00d82463) when the low bit of the flag argument is set. |
