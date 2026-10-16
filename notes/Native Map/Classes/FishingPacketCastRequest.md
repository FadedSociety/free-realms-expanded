# Class: FishingPacketCastRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b62890` | `FishingPacketCastRequest::FishingPacketCastRequest ⚠downgrade` | 🟢 | ctor | Calls the fishing family base constructor FUN_00b60910 with sub-type 6 and stores FishingPacketCastRequest::vftable into the new object. |
| `00b628f0` | `FishingPacketCastRequest::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: restores BaseFishingPacket::vftable and BasePacket::vftable, frees the object with _free when bit 0 of the flag argument is set, and returns the pointer. |
