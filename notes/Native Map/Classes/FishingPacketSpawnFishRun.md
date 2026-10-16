# Class: FishingPacketSpawnFishRun

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b6a590` | `FishingPacketSpawnFishRun::FishingPacketSpawnFishRun` | ✅ | ctor | Calls the base fishing-packet constructor with subtype 0x12, installs the packet vtable, and initializes its string and spawn-info list members. |
| `00b6aeb0` | `ClearAndDeleteSpawnFishRunPacket` | ✅ | dtor | Calls the SpawnFishRun cleanup body and frees packet storage only when flag bit zero is set; otherwise returns the original pointer. |
