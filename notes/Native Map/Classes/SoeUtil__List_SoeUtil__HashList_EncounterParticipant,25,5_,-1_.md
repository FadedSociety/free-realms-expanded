# Class: SoeUtil::List<SoeUtil::HashList<EncounterParticipant,25,5>,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a23180` | `— ⚠reject` | ⚪ | virtual | Node-acquire implementation for the list of participant hash lists: calls the pool helpers 006fb9c0/006fb860 and, when a recycled node is returned, initialises it through virtual slot 4; otherwise (or when the thread po… |
| `00a231c0` | `— ⚠reject` | ⚪ | virtual | Deletes one node of the list: calls the free routine 00401c00 with the node pointer and element size 4 (the raw bytes are mov eax,[esp+4]; push 4; push eax; call FreeMemory). |
| `00a27690` | `ParticipantMapListDtor` | ✅ | dtor | Installs list vtable and destroys each nested participant map until the list is empty. |
| `00a27700` | `ParticipantMapListScalarDeletingDtor` | ✅ | virtual | Destroys nested maps and frees object when deletion flag low bit is set. |
