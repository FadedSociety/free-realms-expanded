# Class: BaseAchievementPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ba97d0` | `BaseAchievementPacket::BaseAchievementPacket` | ✅ | ctor | Stores BasePacket::vftable, writes 0x96 at +4, installs BaseAchievementPacket::vftable, and stores the supplied subtype at +8. |
| `00baa2a0` | `BaseAchievementPacket::~BaseAchievementPacket` | ✅ | dtor | Restores BasePacket::vftable and frees the object only when flag bit 0 is set. |
