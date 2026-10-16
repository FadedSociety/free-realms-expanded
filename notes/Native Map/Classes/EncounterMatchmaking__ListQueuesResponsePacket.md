# Class: EncounterMatchmaking::ListQueuesResponsePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a9f1d0` | `EncounterMatchmaking::ListQueuesResponsePacket::ListQueuesResponsePacket` | ✅ | ctor | Constructor: calls the matchmaking family constructor FUN_00a7f0d0 with 2, stores EncounterMatchmaking::ListQueuesResponsePacket::vftable at +0 and a SoeUtil::List<EncounterMatchmaking::MatchmakingQueueDefinition,-1> vt… |
| `00a9f240` | `EncounterMatchmaking::ListQueuesResponsePacket::~ListQueuesResponsePacket` | 🟢 | dtor | Destructor body: restores the SoeUtil::List vtable at +0x18, then loops calling FUN_00a80850 on the current node at +0x1c until it becomes null (draining the queue-definition list), and finally rewrites the object vtabl… |
| `00aa0680` | `EncounterMatchmaking::ListQueuesResponsePacket::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor: calls the destructor body FUN_00a9f240 (list drain plus base vtable restore) and, when bit 0 of the flags argument is set, calls _free on the object; returns the object pointer. |
