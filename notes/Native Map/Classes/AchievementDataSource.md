# Class: AchievementDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ce37d0` | `AchievementDataSource::ColumnNameByIndex` | ✅ | virtual | Virtual at AchievementDataSource::vftable +0x30 slot [1] (one stack argument): switch on the index returning Id, Title, Description, Time Started, Time Finished, Complete, Objective Count, Members Only, Icon Id, Trophy … |
| `00ce38c0` | `AchievementDataSource::FlagGatedTailCall ?` | 🟡 | virtual | Virtual at AchievementDataSource::vftable +0x30 slot [2] (no stack arguments): when the byte at this+0x22c is zero it calls vtbl+0x38 of the primary object at this-0x30; then tail-jumps to FUN_010c8f30 with ecx = this+4. |
| `00ce38f0` | `AchievementDataSource::FlagGatedCall010c90c0 ?` | 🟡 | virtual | Virtual at AchievementDataSource::vftable +0x34 slot [4] (one stack argument, ret 4): when the byte at this+0x228 is zero it calls vtbl+0x38 of the primary object at this-0x34; then calls FUN_010c90c0 with ecx = this an… |
| `00ce4c30` | `AchievementDataSource::SetTotalPoints` | ✅ | virtual | Virtual slot [14] of AchievementDataSource::vftable (no stack arguments): zeroes the total at this+0x274, walks the node list headed at the object at this+0x258 (+0xc, next link node+0x188), and for each node calls FUN_… |
| `00ce5170` | `AchievementDataSourceEntry::Dtor ?` | 🟡 | dtor | Restores the embedded array vtable, destroys array storage, frees allocation at +0x268, clears it, and calls the embedded holder destructor. |
| `00ce52a0` | `AchievementDataSource::Ctor ?` | 🟡 | ctor | Calls the base constructor, installs source vtables, saves two arguments at indices 0x95/0x96, clears state, and initializes the embedded array fields. |
| `00ce5360` | `AchievementDataSource::ScalarDeletingDtor ?` | 🟡 | dtor | Calls the source destructor body, conditionally frees the object when flag bit 0 is set, then returns its pointer. |
| `00ce5380` | `AchievementDataSource::SelectRow ? ⚠rename` | 🟡 | virtual | Checks selection state and switches on an action/column value; dispatches to string, integer, and nested-row helpers, returning false for unsupported cases. |
