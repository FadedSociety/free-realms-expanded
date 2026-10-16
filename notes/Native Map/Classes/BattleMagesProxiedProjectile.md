# Class: BattleMagesProxiedProjectile

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cd5280` | `BattleMagesProxiedProjectile::~BattleMagesProxiedProjectile` | ✅ | dtor | Stores BattleMagesProxiedProjectile::vftable at this. When the dword at +0x14 is non-zero it clears its +0x1c, sets its +0x20 to -1, calls 00769710, releases the object through 00fd81d0 and clears +0x14. Then calls virt… |
| `00cd52d0` | `— ⚠reject` | ⚪ | ctor | Stores arg2 at +0x10 and BattleMagesProxiedProjectile::vftable at this; zeroes +0x14, +0x18 and +0x1c; stores arg6 at +0x20 and the dword at arg6+4 at +0x24; stores arg3 and arg4 at +0x28 and +0x2c; copies the 16-dword … |
| `00cd5330` | `BattleMagesProxiedProjectile::ScalarDeletingDtor` | ✅ | dtor | Calls destructor body 00cd5280 on this, then frees this when bit 0 of the flag argument is set; returns this. |
