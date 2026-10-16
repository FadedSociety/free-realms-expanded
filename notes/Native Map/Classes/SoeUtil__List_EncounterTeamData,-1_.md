# Class: SoeUtil::List<EncounterTeamData,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a23240` | `— ⚠reject` | ⚪ | virtual | Node-acquire implementation for the encounter team data list: pops a recycled node from the pool (006fb9c0/006fb860) and initialises it via virtual slot 4, otherwise allocates a fresh 0x20-byte, 4-aligned block with 006… |
| `00a23280` | `SoeUtil::ListEncounterTeamDataFreeNode` | ✅ | virtual | Deletes one node of the team list with 00401c00(node, 4). |
| `00a24b30` | `EncounterTeamListDtor` | ✅ | dtor | Installs the list vtable and destroys each node at field index one until empty. |
| `00a24ba0` | `EncounterTeamListScalarDeletingDtor` | ✅ | virtual | Destroys list nodes and frees the object only when deletion flag bit zero is set. |
| `00a267e0` | `AppendEncounterTeamRecord` | 🟢 | function | Constructs a team node, links it after the tail and increments the list count. |
