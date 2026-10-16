# Class: WallOfDataWalletBalancePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00adcf80` | `WallOfDataWalletBalancePacket::WallOfDataWalletBalancePacket` | ✅ | ctor | Base constructor call 00adcd60 with event type 6, then installs WallOfDataWalletBalancePacket::vftable and zeroes the two dwords at +0xc and +0x10. |
| `00add1c0` | `WallOfDataWalletBalancePacket::ScalarDeletingDtor` | ✅ | dtor | Virtual deleting destructor: resets the vtable to BasePacket::vftable inside an SEH frame and frees the object when the flag argument has bit 0 set; it is vtable slot 0 of WallOfDataWalletBalancePacket. |
| `00addc90` | `WallOfDataWalletBalancePacket_Serialize` | 🟢 | function | Writes the wall-of-data header with 00addaa0, then appends the two 4-byte fields at +0xc and +0x10. |
