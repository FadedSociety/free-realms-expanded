# Class: AchievementObjectiveCompletePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00baa900` | `AchievementObjectiveCompletePacket::AchievementObjectiveCompletePacket` | ✅ | ctor | Calls the base achievement constructor with subtype 8, stores three dwords, zeroes another dword, stores a byte, and returns this. |
| `00baa980` | `AchievementObjectiveCompletePacket::~AchievementObjectiveCompletePacket` | ✅ | dtor | Restores BasePacket::vftable and frees the object when flag bit 0 is set. |
