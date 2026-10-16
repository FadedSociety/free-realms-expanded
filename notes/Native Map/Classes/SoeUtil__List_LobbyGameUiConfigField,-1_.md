# Class: SoeUtil::List<LobbyGameUiConfigField,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c1b5c0` | `SoeUtil::List::AllocateNodeStorage` | ✅ | method | Runs a list helper, asks the current memory manager to allocate if available, otherwise asks the global manager, and finally falls back to allocation of 0x1c bytes aligned to four. |
| `00c211d0` | `SoeUtil::ListClearLobbyGameUiConfigField` | ✅ | dtor | Stores the LobbyGameUiConfigField list vtable and repeatedly unlinks and destroys its head node through 00c1fce0. |
| `00c21200` | `LobbyGameUiConfigScalarDeletingDtor` | ✅ | dtor | Unlinks and destroys each list node, then frees the list allocation when flag bit zero is set; otherwise returns the original pointer. |
