# Class: FishingPacketDespawnProxiedFishingSchool

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b60c70` | `— ⚠reject` | ⚪ | ctor | Calls the family base constructor FUN_00b60910 with sub-type 0xc, then stores FishingPacketDespawnProxiedFishingSchool::vftable into the new object and returns it. |
| `00b62bd0` | `FishingPacketDespawnProxiedFishingSchool::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: restores BaseFishingPacket::vftable and BasePacket::vftable, frees the object with _free when bit 0 of the flag argument is set, and returns the pointer. |
