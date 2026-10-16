# Class: GMessageTargetList

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0043ff6f` | `GMessageTargetList::HaveMessageTargets` | ✅ | method | Tiny const predicate: returns true when the next-pointer at this+4 no longer points at this+4, i.e. when the circular target list rooted at this+4 is non-empty. |
