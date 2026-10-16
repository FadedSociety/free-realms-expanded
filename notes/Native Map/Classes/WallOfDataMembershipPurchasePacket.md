# Class: WallOfDataMembershipPurchasePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00adced0` | `WallOfDataMembershipPurchasePacket::WallOfDataMembershipPurchasePacket` | ✅ | ctor | Base constructor call 00adcd60 with event type 5, then installs WallOfDataMembershipPurchasePacket::vftable, zeroes the byte at +0xc and the dword at +0x10. |
| `00add160` | `WallOfDataMembershipPurchasePacket::ScalarDeletingDtor` | ✅ | dtor | Virtual deleting destructor: resets the vtable to BasePacket::vftable inside an SEH frame and frees the object when the flag argument has bit 0 set; it is vtable slot 0 of WallOfDataMembershipPurchasePacket. |
| `00addbf0` | `WallOfDataMembershipPurchasePacket_Serialize` | 🟢 | function | Writes the wall-of-data header with 00addaa0, then appends the 1-byte flag at +0xc and the 4-byte value at +0x10. |
