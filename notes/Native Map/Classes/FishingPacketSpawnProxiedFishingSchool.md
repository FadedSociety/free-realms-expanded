# Class: FishingPacketSpawnProxiedFishingSchool

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b678e0` | `FishingPacketSpawnProxiedFishingSchool::FishingPacketSpawnProxiedFishingSchool` | ✅ | ctor | Calls the base fishing-packet constructor with subtype 0x0b, installs the packet vtable, and initializes its model-entry and integer list members. |
| `00b68ba0` | `FishingPacketSpawnProxiedFishingSchool::~FishingPacketSpawnProxiedFishingSchool` | ✅ | dtor | Calls the packet cleanup body and frees the packet storage only when flag bit zero is set; otherwise returns the original pointer. |
