# Class: AchievementMostRecentDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ce4e30` | `AchievementMostRecentDataSource::SortAndRefresh ?` | 🟡 | method | Collects entries whose fields at +0x18/+0x1c differ from the excluded pair, heap-sorts by +0x1c then +0x18, refreshes up to four rows, and sets +0x25c. |
| `00ce5890` | `AchievementMostRecentDataSource::Ctor ?` | 🟡 | ctor | Constructs the AchievementDataSource base, then installs most-recent source vtables at the base and embedded subobjects. |
| `00ce5960` | `AchievementMostRecentDataSource::ScalarDeletingDtor ?` | 🟡 | dtor | Calls the achievement array cleanup routine, conditionally frees the object when flag bit 0 is set, and returns its pointer. |
