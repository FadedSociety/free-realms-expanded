# Class: FishingPacketSpecialRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b62cb0` | `FishingPacketSpecialRequest::FishingPacketSpecialRequest ? ⚠downgrade` | 🟡 | ctor | Calls the fishing family base constructor FUN_00b60910 with sub-type 0xf (15) and stores FishingPacketSpecialRequest::vftable into the new object. |
| `00b62d10` | `FishingPacketSpecialRequest::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: restores BaseFishingPacket::vftable and BasePacket::vftable, calls _free when bit 0 of the flag argument is set, and returns the pointer. |
