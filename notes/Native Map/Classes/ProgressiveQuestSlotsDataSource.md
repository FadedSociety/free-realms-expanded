# Class: ProgressiveQuestSlotsDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ce7f70` | `ProgressiveQuestSlotsDataSource::ColumnName ?` | 🟡 | virtual | Maps column indices 0..11 to slot/item/key/quest eligibility, key bundle, claim, tooltip, name, and quest id fields. |
| `00ce80b0` | `ProgressiveQuestSlotsDataSource::GetField38 ? → ProgressiveQuestSlotsDataSource…` | 🟡 | virtual | Reads nested quest data pointer and returns its field at offset +0x38, or zero when pointer is null. |
| `00ce85f0` | `— ⚠reject` | ⚪ | virtual | Bounds-checks slot rows and returns slot id, icon, slot/key/quest predicates, key bundle values, progress, tooltip and localized labels. |
| `00ce8b10` | `ProgressiveQuestSlotsDataSource::Ctor ?` | 🟡 | ctor | Constructs ProgressiveQuest.Slots holder, installs slots source vtables, and stores two supplied arguments. |
| `00ce8bf0` | `ProgressiveQuestSlotsDataSource::ScalarDeletingDtor ?` | 🟡 | dtor | Calls the embedded holder destructor, conditionally frees the object when flag bit 0 is set, and returns its pointer. |
