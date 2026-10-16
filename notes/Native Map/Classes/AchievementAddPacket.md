# Class: AchievementAddPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bab710` | `AchievementAddPacket::AchievementAddPacket` | ✅ | ctor | Calls the base achievement constructor with subtype 2, installs AchievementAddPacket::vftable, initializes an embedded object at +0x10 through FUN_0091c9f0, and returns this. |
| `00bab770` | `AchievementAddPacket::~AchievementAddPacket` | ✅ | dtor | Calls the base packet cleanup, conditionally frees this object when flag bit 0 is set, and otherwise returns its pointer. |
