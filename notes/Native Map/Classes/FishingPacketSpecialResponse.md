# Class: FishingPacketSpecialResponse

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b62d80` | `FishingPacketSpecialResponse::FishingPacketSpecialResponse ⚠downgrade` | 🟢 | ctor | Calls the fishing family base constructor FUN_00b60910 with sub-type 0x10 (16) and stores FishingPacketSpecialResponse::vftable into the new object. |
| `00b62de0` | `FishingPacketSpecialResponse::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: restores BaseFishingPacket::vftable and BasePacket::vftable, calls _free when bit 0 of the flag argument is set, and returns the pointer. |
