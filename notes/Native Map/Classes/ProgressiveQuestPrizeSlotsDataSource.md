# Class: ProgressiveQuestPrizeSlotsDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ce8020` | `ProgressiveQuestPrizeSlotsDataSource::ColumnName ?` | 🟡 | virtual | Maps column indices 0..6 to prize slot, icon, progress, claim eligibility, tooltip, name, and quest id fields. |
| `00ce80d0` | `ProgressiveQuestPrizeSlotsDataSource::GetField48 ? → ProgressiveQuestPrizeSlots…` | 🟡 | virtual | Reads nested quest data pointer and returns its field at offset +0x48, or zero when pointer is null. |
| `00ce8860` | `— ⚠reject` | ⚪ | virtual | Bounds-checks prize rows and returns prize slot ids, icon, progress, claim predicate, tooltip/name strings, and quest id. |
| `00ce8c50` | `ProgressiveQuestPrizeSlotsDataSource::Ctor ?` | 🟡 | ctor | Constructs ProgressiveQuest.PrizeSlots holder, installs prize slots source vtables, and stores two supplied arguments. |
| `00ce8d30` | `ProgressiveQuestPrizeSlotsDataSource::ScalarDeletingDtor ?` | 🟡 | dtor | Calls the embedded holder destructor, conditionally frees the object when flag bit 0 is set, and returns its pointer. |
