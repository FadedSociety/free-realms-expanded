# Class: AchievementObjectiveAddedPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bab790` | `— ⚠reject` | ⚪ | ctor | Calls the base achievement constructor with subtype 5, installs its packet vtable, stores an argument at +0xc, initializes an embedded object at +0x18 through FUN_0091c8b0, and returns this. |
| `00bab800` | `AchievementObjectiveAddedPacket::~AchievementObjectiveAddedPacket` | ✅ | dtor | Calls the base cleanup, conditionally frees the object when flag bit 0 is set, and otherwise returns this. |
