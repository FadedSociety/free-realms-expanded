# Class: MiniGameBossUpdatePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c47e80` | `MiniGameBossUpdatePacket_Construct` | ✅ | method | Constructs the boss-update packet: runs the boss base constructor with discriminator 1 (stored at +0x14), installs MiniGameBossUpdatePacket::vftable and zeroes the three payload dwords at +0x18/+0x1c/+0x20; the S0393 ha… |
| `00c47ef0` | `MiniGameBossUpdatePacket_Destruct` | 🟢 | dtor | Destructor body of the boss-update packet temporary: writes BasePacket::vftable into the object at +0 and returns; the handler 00c48c90 calls it at the end of the update branch and an EH unwind funclet references it. |
| `00c48130` | `MiniGameBossUpdatePacket_ScalarDeletingDtor` | 🟢 | virtual | Virtual slot 0 deleting destructor: writes BasePacket::vftable into the object and frees it with _free when the low bit of the flag argument is set. |
