# Class: ProxiedVehicleRaceExplosion

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b9dc70` | `ProxiedVehicleRaceExplosionDeletingDtor` | 🟢 | dtor | MSVC scalar deleting destructor for a ProxiedVehicleRaceExplosion: calls FUN_00ce0dd0(this) to destroy the object contents and, when the delete flag is set, calls _free(this); returns this. |
| `00ce0dd0` | `ProxiedVehicleRaceExplosion::~ProxiedVehicleRaceExplosion` | ✅ | dtor | Stores ProxiedVehicleRaceExplosion::vftable at this; if [this+8] is nonnull calls vtbl[0] of that object with 1; if [this+0x40] is nonzero calls FUN_007d01f0, calls FUN_007d06b0(0) and nulls [this+0x40]; then stores Ass… |
| `00ce0ee0` | `ProxiedVehicleRaceExplosion::ScalarDeletingDtor` | ✅ | dtor | Thiscall scalar deleting destructor (ret 4): calls the body 00ce0dd0, then calls _free(this) when bit 0 of its stack argument is set; returns this in eax. |
| `00ce0f00` | `ProxiedVehicleRaceExplosion::ProxiedVehicleRaceExplosion` | ✅ | ctor | Thiscall constructor with two stack arguments: stores the AssetRequester vtable, stores the first argument at +4, then stores ProxiedVehicleRaceExplosion::vftable at this; zeroes +8, +0x38, the byte at +0x14, +0x3c and … |
| `00ce11f0` | `— ⚠reject` | ⚪ | virtual | Virtual slot [3] of ProxiedVehicleRaceExplosion::vftable (no stack argument): allocates 0x68 bytes with FUN_00d833a6 and, when allocation succeeds, calls FUN_0071bed0 with the dword at this+4 and the dword at the FUN_00… |
