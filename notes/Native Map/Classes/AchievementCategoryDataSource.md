# Class: AchievementCategoryDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ce5a30` | `AchievementCategoryDataSource::Ctor ?` | 🟡 | ctor | Constructs the BaseClient.Achievements.Categories holder, installs the category source vtable, stores two arguments, and sets the initialized byte. |
| `00ce5c30` | `AchievementCategoryDataSource::DtorBody ?` | 🟡 | dtor | Restores category source vtable, releases child pointer at +0x30 if present, sets initialized byte, and destroys its embedded base holder. |
| `00ce5de0` | `AchievementCategoryDataSource::ScalarDeletingDtor ?` | 🟡 | dtor | Calls category source destructor body, conditionally frees the object when flag bit 0 is set, and returns its pointer. |
