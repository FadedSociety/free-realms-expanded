# Class: PlayerUpdatePacketLootEvent

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008d0a00` | `PlayerUpdatePacketLootEvent::PlayerUpdatePacketLootEvent` | ✅ | ctor | Constructor of the LootEvent player-update packet: calls the family constructor FUN_0089fbb0 with sub-opcode 0x1d (29), installs PlayerUpdatePacketLootEvent::vftable, then initializes one embedded SoeUtil::IString<char>… |
| `008d0a80` | `PlayerUpdatePacketLootEvent::~PlayerUpdatePacketLootEvent` | 🟢 | dtor | Destructor body of the LootEvent player-update packet: rewrites the embedded SoeUtil::IString<char> member at +0x30 (param_1[0xc]) to its vtable, LOCK-decrements the shared refcount stored immediately before the charact… |
| `008dcc90` | `PlayerUpdatePacketLootEvent::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: runs the destructor body 008d0a80 and frees the object with _free (00d82463) when the low bit of the flag argument is set. |
