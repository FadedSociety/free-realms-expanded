# Class: FishingPacketSpawnProxiedFishingBobber

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b62a30` | `FishingPacketSpawnProxiedFishingBobber::FishingPacketSpawnProxiedFishingBobber` | ✅ | ctor | Calls the fishing family base constructor FUN_00b60910 with sub-type 8 and stores FishingPacketSpawnProxiedFishingBobber::vftable into the new object. |
| `00b62a90` | `FishingPacketSpawnProxiedFishingBobber::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: restores BaseFishingPacket::vftable and BasePacket::vftable, calls _free when bit 0 of the flag argument is set, and returns the pointer. |
| `00b66a00` | `FishingPacketSpawnProxiedFishingBobber_Read ⚠downgrade` | 🟢 | method | Reads the sub-8 payload: header via FUN_00b66830, the 8-byte handle into +0x10/+0x14, a dword into +0x18, then two four-float vectors into +0x20 and +0x30 through FUN_008e2410. |
