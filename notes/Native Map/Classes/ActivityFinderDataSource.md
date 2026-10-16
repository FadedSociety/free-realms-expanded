# Class: ActivityFinderDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ceaab0` | `ActivityFinderDataSource::ColumnName ?` | 🟡 | virtual | Maps indices 0..7 to ProfileId, HasQuest, IconId, BadgeId, QuestId, QuestTitle, QuestDescription, and ProfileRank. |
| `00ceab30` | `— ⚠reject` | ⚪ | virtual | Jumps to 00be02c0, which returns the dword at this+0x18ccc. |
| `00ceab40` | `— ⚠reject` | ⚪ | virtual | Bounds-checks the requested row and returns profile id, quest presence, icon, badge, quest id/title/description, or resolved profile rank. |
| `00ceacf0` | `ActivityFinderDataSource::DtorBody ?` | 🟡 | dtor | Restores activity finder source vtables on main object and embedded bases, then destroys embedded base holder. |
| `00cead60` | `ActivityFinderDataSource::Ctor ?` | 🟡 | ctor | Constructs BaseClient.ActivityFinder holder, installs source vtables, and stores its two supplied arguments. |
| `00ceade0` | `ActivityFinderDataSource::ScalarDeletingDtor ?` | 🟡 | dtor | Calls activity finder destructor body, conditionally frees the object when flag bit 0 is set, and returns its pointer. |
