# Class: FishingPacketUpdateProxiedFishingBobber

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b62b00` | `— ⚠reject` | ⚪ | ctor | Calls the fishing family base constructor FUN_00b60910 with sub-type 10 and stores FishingPacketUpdateProxiedFishingBobber::vftable into the new object. |
| `00b62b60` | `FishingPacketUpdateProxiedFishingBobber::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: restores BaseFishingPacket::vftable and BasePacket::vftable, calls _free when bit 0 of the flag argument is set, and returns the pointer. |
| `00b66a80` | `FishingPacketUpdateProxiedFishingBobber_Read` | ✅ | method | Reads the sub-10 payload: header via FUN_00b66830, the 8-byte handle into +0x10/+0x14, a dword into +0x18, then two single bytes converted to bools at +0x1c and +0x1d. |
