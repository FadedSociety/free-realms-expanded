# Class: MiniGameBossDeletePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c47f70` | `MiniGameBossDeletePacket_Construct` | ✅ | method | Constructs the boss-delete packet: runs the boss base constructor with discriminator 2 (stored at +0x14), installs MiniGameBossDeletePacket::vftable and stores the caller's id argument at +0x18; the S0393 handler 00c48c… |
| `00c47fe0` | `MiniGameBossDeletePacket_Destruct` | 🟢 | dtor | Destructor body of the boss-delete packet temporary: writes BasePacket::vftable into the object at +0 and returns; called from the delete branch of the handler 00c48c90 and referenced by an EH unwind funclet. |
| `00c48190` | `MiniGameBossDeletePacket_ScalarDeletingDtor` | 🟢 | virtual | Virtual slot 0 deleting destructor: writes BasePacket::vftable into the object and frees it with _free when the low bit of the flag argument is set. |
