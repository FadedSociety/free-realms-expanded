# Class: FishingPacketUpdateData

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b62620` | `— ⚠reject` | ⚪ | ctor | Calls the fishing family base constructor FUN_00b60910 with sub-type 1 and then stores FishingPacketUpdateData::vftable into the new object. |
| `00b62680` | `FishingPacketUpdateData::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: restores BaseFishingPacket::vftable and BasePacket::vftable and calls _free(param_1) when bit 0 of the flag argument is set, returning the object pointer. |
| `00b66890` | `FishingPacketUpdateData_Read` | ✅ | method | Reads the sub-1 payload into the packet object: first the header via FUN_00b66830, then the 8-byte handle into +0x10/+0x14 (zeroing both and flagging end-of-stream when fewer than two dwords remain), then a four-float v… |
