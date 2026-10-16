# Class: AchievementCompletePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ba98c0` | `AchievementCompletePacket::AchievementCompletePacket` | ✅ | ctor | Calls the base achievement constructor with subtype 3, installs its packet vtable, stores two dwords and one byte from arguments, then returns this. |
| `00baa300` | `AchievementCompletePacket::~AchievementCompletePacket` | ✅ | dtor | Restores BasePacket::vftable and frees the object when flag bit 0 is set. |
