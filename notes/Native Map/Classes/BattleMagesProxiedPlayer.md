# Class: BattleMagesProxiedPlayer

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cd4d50` | `BattleMagesProxiedPlayer::~BattleMagesProxiedPlayer` | ✅ | dtor | Stores BattleMagesProxiedPlayer::vftable at [ECX] (mov dword [ecx],0x0183972c; ret). Nothing else. |
| `00cd4e70` | `BattleMagesProxiedPlayer::BattleMagesProxiedPlayer` | ✅ | ctor | Stores BattleMagesProxiedPlayer::vftable at this; stores arg2 at +0x8 and arg3 at +0xc; stores arg4 (byte) at +0x14; zeroes +0x10, +0x18 and +0x1c; then loads the dword at (result of 00769710)+4 into +0x1c; returns this. |
| `00cd4eb0` | `BattleMagesProxiedPlayer::SetVtableThenFreeIfFlag` | ✅ | virtual | Virtual slot 0: stores BattleMagesProxiedPlayer::vftable at this, frees this when bit 0 of the flag argument is set, and returns this. The destructor body 00cd4d50 is not called here. |
