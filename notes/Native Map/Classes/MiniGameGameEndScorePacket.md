# Class: MiniGameGameEndScorePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009b61a0` | `MiniGameGameEndScorePacketCtor` | ✅ | ctor | Calls base minigame packet constructor with sub-opcode 0x2f, installs concrete vtable, and initializes an empty MiniGameScoreInfo array. |
| `009b8610` | `MiniGameGameEndScorePacket::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor for the 39/47 end-score packet (MiniGameGameEndScorePacket): calls the class destructor body 009b6220, which destroys the SoeUtil::Array<MiniGameScoreInfo> member at +0x18 and restores BasePac… |
