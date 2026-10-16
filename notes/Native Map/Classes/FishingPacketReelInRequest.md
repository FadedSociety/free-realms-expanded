# Class: FishingPacketReelInRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b62960` | `FishingPacketReelInRequest::FishingPacketReelInRequest ⚠downgrade` | 🟢 | ctor | Calls the fishing family base constructor FUN_00b60910 with sub-type 7 and stores FishingPacketReelInRequest::vftable into the new object. |
| `00b629c0` | `FishingPacketReelInRequest::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: restores BaseFishingPacket::vftable and BasePacket::vftable, frees the object with _free when bit 0 of the flag argument is set, and returns the pointer. |
| `00b66990` | `FishingPacketReelInRequest_Read` | ✅ | method | Reads the sub-7 payload: header via FUN_00b66830, the 8-byte handle into +0x10/+0x14, then one byte into +0x18 converted to a bool, with the same short-read handling (zero field, end flag, clamped cursor). |
