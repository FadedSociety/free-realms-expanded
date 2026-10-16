# Class: FishingPacketCastAnimRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b627c0` | `FishingPacketCastAnimRequest::FishingPacketCastAnimRequest ⚠downgrade` | 🟢 | ctor | Calls the fishing family base constructor FUN_00b60910 with sub-type 5 and stores FishingPacketCastAnimRequest::vftable into the new object. |
| `00b62820` | `FishingPacketCastAnimRequest::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: restores BaseFishingPacket::vftable and BasePacket::vftable, calls _free on the object when bit 0 of the flag argument is set, and returns the pointer. |
| `00b668f0` | `FishingPacketCastAnimRequest_Read` | ✅ | method | Reads the sub-5 payload: header via FUN_00b66830, the 8-byte handle into +0x10/+0x14, a float into +0x18 (setting the NaN flag at param_2+0x11 when it is not a number) and a four-float vector into +0x20 through FUN_008e… |
