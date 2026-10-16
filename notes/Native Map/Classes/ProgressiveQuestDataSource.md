# Class: ProgressiveQuestDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ce7ea0` | `ProgressiveQuestDataSource::ColumnName ?` | 🟡 | virtual | Maps column indices 0..11 to progressive quest id/name/timer/slot/icon/objective/window/button fields; returns <error> otherwise. |
| `00ce7f50` | `ProgressiveQuestDataSource::GetHasQuest ? ⚠rename` | 🟡 | virtual | Returns whether the quest data pointer at +0x228 has a nonzero field at offset +0x10. |
| `00ce8460` | `ProgressiveQuestDataSource::GetColumn ?` | 🟡 | virtual | Reads the current quest record and returns quest id, localized name, timer, active slot, icon, objective strings, window flag, and button string ids for supported columns. |
| `00ce89d0` | `ProgressiveQuestDataSource::Ctor ?` | 🟡 | ctor | Constructs ProgressiveQuest.Definition holder, installs source vtables, and stores two supplied arguments. |
| `00ce8ab0` | `ProgressiveQuestDataSource::ScalarDeletingDtor ?` | 🟡 | dtor | Calls the embedded holder destructor, conditionally frees the object when flag bit 0 is set, and returns its pointer. |
